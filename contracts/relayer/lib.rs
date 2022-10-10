#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

pub mod constants;

#[ink::contract]
pub mod relayer {
    use crate::constants::{IV, SCALAR_FIELD, ZEROS};
    use ink_prelude::{string::String, vec, vec::Vec};
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use scale::{Decode, Encode};
    // use zkmega_arkworks::{curve::Bn254, groth16};
    use zkp_u256::U256;
    #[ink(event)]
    pub struct Deposit {
        commitment: String,
        leaf_index: u32,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct Withdrawal {
        recipient: AccountId,
        nullifier_hash: String,
        relayer: AccountId,
        fee: u128,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        AlreadySubmitted,
        VerifyCatchErr,
        MerkleTreeFull,
        IndexOutOfBounds,
        RootNotExist,
        AlreadySpent,
        InvalidWithdrawProof,
        VerifyFailed,
        BadLength
    }

    const ROOT_HISTORY_SIZE: u32 = 30; // merkle tree history size
    const DEPOSIT_AMOUNT: Balance = 1; // required deposit amount

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Relayer {
        // Stores the ZK result
        pub verifier: AccountId,
        pub commitments: Mapping<String, bool>,
        pub nullifier_hashes: Mapping<String, bool>,
        pub levels: u32,
        pub filled_subtrees: Mapping<u32, String>,
        pub roots: Mapping<u32, String>,
        pub current_root_index: u32,
        pub next_index: u32,
    }

    impl Relayer {
        #[ink(constructor)]
        pub fn new(levels: u32, verifier: AccountId) -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, levels, verifier);
            })
        }

        fn new_init(&mut self, levels: u32, verifier: AccountId) {
            self.levels = levels;
            self.verifier = verifier;
            for i in 0..levels {
                self.filled_subtrees
                    .insert(i, &String::from(ZEROS[i as usize]));
            }
            self.roots
                .insert(0, &String::from(ZEROS[(levels - 1) as usize]));
        }

        #[ink(message, payable)]
        pub fn deposit(&mut self, commitment: String) -> Result<u32, Error> {
            if self.commitments.contains(commitment.clone()) {
                return Err(Error::AlreadySubmitted);
            }
            // Detect transferred token amount
            assert!(self.env().transferred_value() == DEPOSIT_AMOUNT, "invalid deposit amount!");

            let inserted_index = self.insert(commitment.clone())?;
            self.commitments.insert(commitment.clone(), &true);
            Self::env().emit_event(Deposit {
                commitment,
                leaf_index: inserted_index,
                timestamp: Self::env().block_timestamp(),
            });
            Ok(inserted_index)
        }

        #[ink(message)]
        pub fn withdraw(
            &mut self,
            proof: String,
            root: String,
            nullifier_hash: String,
            recipient: AccountId,
            relayer: AccountId,
            fee: u128,
            refund: u128,
        ) -> Result<(), Error> {
            if !self.is_known_root(root.clone()) {
                return Err(Error::RootNotExist);
            }
            if self.nullifier_hashes.contains(nullifier_hash.clone()) {
                return Err(Error::AlreadySpent);
            }
            if self.verifier == AccountId::from([0;32]) {
              return Err(Error::VerifyFailed);
            }
            
            // function verify() of contract verifier, copied from target/ink/metadata.json after contract verifier compiled
            let selector: [u8; 4] = [0x18, 0x60, 0xff, 0x3b];
            let verify_result: bool =
                ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                    .call_type(
                        ink_env::call::Call::new()
                            .callee(self.verifier)
                            .gas_limit(0)
                            .transferred_value(0),
                    )
                    .exec_input(
                        ink_env::call::ExecutionInput::new(ink_env::call::Selector::new(selector))
                            .push_arg(proof)
                            .push_arg(root)
                            .push_arg(nullifier_hash.clone())
                            .push_arg(recipient)
                            .push_arg(relayer)
                            .push_arg(fee)
                            .push_arg(refund),
                    )
                    .call_flags(ink_env::CallFlags::default().set_allow_reentry(true))
                    .returns::<bool>()
                    .fire().unwrap();
            if !verify_result {
                return Err(Error::VerifyFailed);
            }
            self.nullifier_hashes.insert(nullifier_hash.clone(), &true);
            // process_withdraw(recipient, relayer, fee, refund);
            Self::env().emit_event(Withdrawal {
                recipient,
                nullifier_hash,
                relayer,
                fee,
            });
            Ok(())
        }

        /// Whether the root is present in the root history
        #[ink(message)]
        pub fn is_known_root(&self, root: String) -> bool {
            for i in (0..self.current_root_index + 1).rev() {
                if i == 0 {
                    return false;
                }
                if root == self.roots.get(i).unwrap() {
                    return true;
                }
            }
            false
        }

        pub fn mimc_sponge(&self, inputs: Vec<String>) -> U256 {
            let p = U256::from_decimal_str(SCALAR_FIELD).unwrap();
            let mut left = U256::ZERO;
            let mut right = U256::ZERO;
            let mut t;
            let mut a;
            let k = U256::ZERO;
            for elt in inputs {
                left = left + U256::from_hex_str(&elt) % &p;
                for i in 0..(220 - 1) {
                    t = (&left + U256::from_decimal_str(IV[i]).unwrap() + &k) % &p;
                    a = t.mulmod(&t, &p); // t^2
                    let l_new = (a.mulmod(&a, &p).mulmod(&t, &p) + right) % &p;
                    right = left.clone();
                    left = l_new;
                    // ink_env::debug_println!("hash: {}", left.to_decimal_string());
                }
                t = (&k + &left) % &p;
                a = t.mulmod(&t, &p); // t^2
                right = (a.mulmod(&a, &p).mulmod(&t, &p) + right) % &p; // t^5
            }
            // ink_env::debug_println!("hash: {}", left.to_hex_string());
            left
        }

        pub fn insert(&mut self, leaf: String) -> Result<u32, Error> {
            // self.next_index = nextndex;
            let next_index = self.next_index;
            if next_index >= u32::pow(2, 10) {
                return Err(Error::MerkleTreeFull);
            }
            let mut current_index = next_index;
            let mut current_level_hash = leaf;
            let mut left: String;
            let mut right: String;
            for i in 0..self.levels {
                if current_index % 2 == 0 {
                    left = current_level_hash.clone();
                    right = String::from(ZEROS[i as usize]);
                    self.filled_subtrees.insert(i, &current_level_hash);
                } else {
                    left = self.filled_subtrees.get(i).unwrap();
                    right = current_level_hash.clone();
                }
                // current_level_hash = self.hash_left_right(left, right);
                current_level_hash = self
                    .mimc_sponge(vec![left, right])
                    .to_hex_string()
                    .replace("0x", "");
                current_index /= 2;
            }
            let new_root_index = (self.current_root_index + 1) % ROOT_HISTORY_SIZE;
            self.current_root_index = new_root_index;
            self.roots.insert(new_root_index, &current_level_hash);
            self.next_index = next_index + 1;
            Ok(next_index)
        }
    }

    #[cfg(test)]
    mod tests {

        /// Imports all the definitions from the outer scope so we can use them here.
        // The test environment does not support elliptic curve APIs, so public keys and signatures have to be hard-coded for test purposes.
        use super::*;
        use ink_env::{
            self,
            test::{self, default_accounts},
            DefaultEnvironment,
        };
        use ink_lang as ink;
        const ROOT: &str = "222eddf0a52aada170d89dd492bf939c6430d4e10c0bf2b843e6bde7ac46781f";
        const COMMITMENT: &str = "078af378072d903d63ea375d80429a2c89f8cc80a5a3b543ecb22734a090e0c4";
        const NULLIFIER_HASH: &str =
            "15bd4d1ea3140c2a717b781050a6dd46f93a056f8a7e2f40cfd30740a2444a95";
        const PROOF: &str = "637a7593f81167bfd1de206da605e4454ed04c5b13c4086bf64ee2f49b172a0d442e74a38cecb3fb5ed5bf3aec01cdfb3e888b3001818122c479f0cf9947be0e004d488c687a6f8848464f29188e5d8ed3ec06c9a9be660cd542120e15f4f4ac1df265c5d1b823e08e03405f5c98124ea99106171d1a25403eece9223d84bc9c096248c23f97e6d8dff366346ed1291156882813bb88ad87552e04f6e8b0174d0f1967a8ce3406b897b21eee16a0892d8a70cf9f7a53470f95fdf792fef4e983290023ffc6cf4a55cf665b5cc05af0df6f00de98ca7ee7f7e81da6fea2a800dc0f27ed497a5dae7e1040db883cdbaaa3c7ab083cafe408954044ad54e0acd311602f00";

        #[ink::test]
        fn test_deposit() {
            let accounts = default_accounts::<DefaultEnvironment>();

            // Payable
            let mut relayer = Relayer::new(10, AccountId::from([0;32]));            
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            test::set_balance::<DefaultEnvironment>(accounts.alice, 10);
            test::set_value_transferred::<DefaultEnvironment>(1);
            
            // Init commitment
            let commitment = String::from(COMMITMENT);
            let root = String::from(ROOT);

            relayer.deposit(commitment).unwrap();
            assert!(relayer.is_known_root(root));
        }

        #[ink::test]
        fn test_withdraw() {
            let mut relayer = Relayer::new(10, AccountId::from([0;32]));
            let proof: String = String::from(PROOF);
            let root: String = String::from(ROOT);
            let nullifier_hash: String = String::from(NULLIFIER_HASH);
            // the recipient SS58Address is "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            let recipient = AccountId::from([
                212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44,
                133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
            ]);

            // the relayer_account SS58Address is "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
            let relayer_account = AccountId::from([
                142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97,
                54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72,
            ]);
            let fee = 1000000000u128;
            let refund = 2000000000u128;
            let commitment = String::from(COMMITMENT);

            // Payable
            let accounts = default_accounts::<DefaultEnvironment>();
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            test::set_balance::<DefaultEnvironment>(accounts.alice, 10);
            test::set_value_transferred::<DefaultEnvironment>(1);

            relayer.deposit(commitment).unwrap();
            assert_eq!(relayer
                .withdraw(
                    proof,
                    root,
                    nullifier_hash.clone(),
                    recipient,
                    relayer_account,
                    fee,
                    refund,
                ), Err(Error::VerifyFailed));
            assert_eq!(relayer.nullifier_hashes.contains(nullifier_hash), false);
        }

        #[ink::test]
        fn mimc_sponge() {
            // let inputs = vec![U256::ZERO.to_hex_string(), U256::ZERO.to_hex_string()];
            let inputs = vec![String::from(
                "471424a3bb441fde5e66071c0d74bac794d700cb8dbb8f1a996360870bc6ae",
            )];
            let relayer = Relayer::new(10, AccountId::from([0;32]));
            // println!("pow: {}", u64::pow(2, 10));
            relayer.mimc_sponge(inputs);
        }
    }
}
