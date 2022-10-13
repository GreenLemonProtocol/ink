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
        BadLength,
        ThirdContractExecutionFailed,
        InvalidContractAddress,
        WithdrawFailed,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Param {
        TokenId(u32),
        AccountId(AccountId),
        String(String),
    }

    impl Param {
        pub fn get_value<T: Value>(&self) -> Option<T::Type> {
            T::get_value(self)
        }
    }

    pub trait Value {
        type Type;
        fn get_value(type_value: &Param) -> Option<Self::Type>;
    }

    impl Value for u32 {
        type Type = u32;
        fn get_value(type_value: &Param) -> Option<Self::Type> {
            if let Param::TokenId(val) = type_value {
                Some(*val)
            } else {
                None
            }
        }
    }

    impl Value for String {
        type Type = String;
        fn get_value(type_value: &Param) -> Option<Self::Type> {
            if let Param::String(val) = type_value.clone() {
                Some(val)
            } else {
                None
            }
        }
    }

    impl Value for ink_env::AccountId {
        type Type = AccountId;
        fn get_value(type_value: &Param) -> Option<Self::Type> {
            if let Param::AccountId(val) = type_value {
                Some(*val)
            } else {
                None
            }
        }
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum NFTFunction {
        Approve,
        RegisterPublicKeys,
        Transfer,
        TransferFrom,
        Mint,
        Burn,
    }

    const ROOT_HISTORY_SIZE: u32 = 30; // merkle tree history size
    const DEPOSIT_AMOUNT: Balance = 1; // required deposit amount

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Relayer {
        // Stores the ZK result
        pub verifier: AccountId,
        pub erc721: AccountId,
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
        pub fn new(levels: u32, verifier: AccountId, erc721: AccountId) -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, levels, verifier, erc721);
            })
        }

        fn new_init(&mut self, levels: u32, verifier: AccountId, erc721: AccountId) {
            self.levels = levels;
            self.verifier = verifier;
            self.erc721 = erc721;
            for i in 0..levels {
                self.filled_subtrees
                    .insert(i, &String::from(ZEROS[i as usize]));
            }
            self.roots
                .insert(0, &String::from(ZEROS[(levels - 1) as usize]));
        }

        /// deposit coin into contract
        #[ink(message, payable)]
        pub fn deposit(&mut self, commitment: String) -> Result<u32, Error> {
            if self.commitments.contains(commitment.clone()) {
                return Err(Error::AlreadySubmitted);
            }
            // Detect transferred token amount
            assert!(
                self.env().transferred_value() == DEPOSIT_AMOUNT,
                "invalid deposit amount!"
            );

            let inserted_index = self.insert(commitment.clone())?;
            self.commitments.insert(commitment.clone(), &true);
            Self::env().emit_event(Deposit {
                commitment,
                leaf_index: inserted_index,
                timestamp: Self::env().block_timestamp(),
            });
            Ok(inserted_index)
        }

        /// withdraw note to user
        #[ink(message)]
        pub fn withdrawal(
            &mut self,
            proof: String,
            root: String,
            nullifier_hash: String,
            recipient: AccountId,
            relayer: AccountId,
            fee: u128,
            refund: u128,
        ) -> Result<(), Error> {
            return self.withdraw(proof, root, nullifier_hash, recipient, relayer, fee, refund);
        }

        /// execute specified function of erc721 contract, and transfer note to relayer as transaction fees
        #[ink(message)]
        pub fn execute(
            &mut self,
            proof: String,
            root: String,
            nullifier_hash: String,
            recipient: AccountId,
            relayer: AccountId,
            fee: u128,
            refund: u128,
            function: NFTFunction,
            selector: [u8; 4],
            params: Vec<Param>,
        ) -> Result<(), Error> {
            if self
                .withdraw(proof, root, nullifier_hash, recipient, relayer, fee, refund)
                .is_err()
            {
                return Err(Error::WithdrawFailed);
            }

            if self.erc721 == AccountId::from([0; 32]) {
                return Err(Error::InvalidContractAddress);
            }

            let contract = self.erc721;

            // match function of erc721 contract 
            match function {
                NFTFunction::Approve | NFTFunction::Transfer => {
                    let to = params[0].get_value::<AccountId>().unwrap();
                    let id = params[1].get_value::<u32>().unwrap();
                    let ephemeral_public_key = params[2].get_value::<String>().unwrap();
                    let signature = params[3].get_value::<String>().unwrap();
                    crate::call!(contract, selector, to, id, ephemeral_public_key, signature)
                        .returns::<()>()
                        .fire()
                        .unwrap();
                }
                NFTFunction::RegisterPublicKeys => {
                    let alias = params[0].get_value::<String>().unwrap();
                    let scan_public_key = params[1].get_value::<String>().unwrap();
                    let spend_public_key = params[2].get_value::<String>().unwrap();
                    crate::call!(contract, selector, alias, scan_public_key, spend_public_key)
                        .returns::<()>()
                        .fire()
                        .unwrap();
                }
                NFTFunction::TransferFrom => {
                    let from = params[0].get_value::<String>().unwrap();
                    let to = params[1].get_value::<AccountId>().unwrap();
                    let id = params[2].get_value::<u32>().unwrap();
                    let ephemeral_public_key = params[3].get_value::<String>().unwrap();
                    let signature = params[4].get_value::<String>().unwrap();
                    crate::call!(
                        contract,
                        selector,
                        from,
                        to,
                        id,
                        ephemeral_public_key,
                        signature
                    )
                    .returns::<()>()
                    .fire()
                    .unwrap();
                }
                NFTFunction::Mint => {
                    // owner: AccountId, ephemeral_public_key: String
                    let owner = params[0].get_value::<AccountId>().unwrap();
                    let ephemeral_public_key = params[1].get_value::<String>().unwrap();
                    crate::call!(contract, selector, owner, ephemeral_public_key)
                        .returns::<()>()
                        .fire()
                        .unwrap();
                }
                NFTFunction::Burn => {
                    //id: TokenId, signature: String
                    let id = params[0].get_value::<u32>().unwrap();
                    let signature = params[1].get_value::<String>().unwrap();
                    crate::call!(contract, selector, id, signature)
                        .returns::<()>()
                        .fire()
                        .unwrap();
                }
            };
            Ok(())
        }

        /// Withdraw token from contract, and nullifier the note
        fn withdraw(
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
            if self.verifier == AccountId::from([0; 32]) {
                return Err(Error::InvalidContractAddress);
            }

            // The selector of function verify() from contract verifier, copied from target/ink/metadata.json after contract verifier compiled
            // selector = 0x1860ff3b
            let selector: [u8; 4] = [0x18, 0x60, 0xff, 0x3b];
            let contract = self.verifier;
            let verify_result = crate::call!(
                contract,
                selector,
                proof,
                root,
                nullifier_hash.clone(),
                recipient,
                relayer,
                fee,
                refund
            )
            .returns::<bool>()
            .fire()
            .unwrap();
            if !verify_result {
                return Err(Error::VerifyFailed);
            }
            
            // nullifier hash
            self.nullifier_hashes.insert(nullifier_hash.clone(), &true);

            // transfer token to recipient and relayer
            self.process_transfer(recipient, relayer, fee, refund);

            Self::env().emit_event(Withdrawal {
                recipient,
                nullifier_hash,
                relayer,
                fee,
            });
            Ok(())
        }

        /// Transfer token to relayer and recipient
        fn process_transfer(
            &mut self,
            recipient: AccountId,
            relayer: AccountId,
            fee: u128,
            refund: u128,
        ) -> bool {
            if self.env().transfer(relayer, fee).is_err() {
                panic!("contract does not have sufficient free funds")
            }

            if self.env().transfer(recipient, refund).is_err() {
                panic!("contract does not have sufficient free funds")
            }
            true
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

        /// use mimc sponge to hash params
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

        /// insert new leaf to merkle tree
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
        const ROOT: &str = "1b68d520e3f0594a45d3b1ee6cff14962272b66547a218053beb57b07bf36bc4";
        const COMMITMENT: &str = "5ad3068cfac36d516b1f4844ee0885940c670d5f4cdd9ff7826235476fdde3b";
        const NULLIFIER_HASH: &str =
            "299a100c01c4e16cc745af1091fb77e36621a24b36edec50cb7d1aee8c54304b";
        const PROOF: &str = "881bc81785063689cea604fe6df802ffbad64344db5711e8b40b4ce5f7b001068189bbb27a9c980ba66d1c90d955849ea2eed93356e64819bd9f813c2481d510000a4dc82dbdda164e7a6931f02f7b59abcad786b4a081f0aca38e24beec92293017729df011542417d0bf8a18d93a4973fc78a2b61817aff346dc766c5d6d231bd5d3aa7e083815c9b0a9f3047c20aaa89f34f8b16d7e183d108ece6f92ed871f33bcf82fc1c75ca5319e26ef117261ce02dc3f133a9acfc2ad73d7008690832800cc5e9c949bf0d1a2ccb5b45419b21c749af5d163d10059b6662a1ae7c98ec82ad34d3ac58810f5ae7f27dfcaf0e4bdbbe0f50fd7c396845bf2d76f03363a8c0f00";

        #[ink::test]
        fn test_deposit() {
            let accounts = default_accounts::<DefaultEnvironment>();

            // Payable
            let mut relayer = Relayer::new(10, AccountId::from([0; 32]), AccountId::from([0; 32]));
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
        fn test_withdrawal() {
            let mut relayer = Relayer::new(10, AccountId::from([0; 32]), AccountId::from([0; 32]));
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
            let fee = 500000000u128;
            let refund = 500000000u128;
            let commitment = String::from(COMMITMENT);

            // Payable
            let accounts = default_accounts::<DefaultEnvironment>();
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            test::set_balance::<DefaultEnvironment>(accounts.alice, 10);
            test::set_value_transferred::<DefaultEnvironment>(1);

            relayer.deposit(commitment).unwrap();
            assert_eq!(
                relayer.withdrawal(
                    proof,
                    root,
                    nullifier_hash.clone(),
                    recipient,
                    relayer_account,
                    fee,
                    refund,
                ),
                Err(Error::InvalidContractAddress)
            );
            assert_eq!(relayer.nullifier_hashes.contains(nullifier_hash), false);
        }

        #[ink::test]
        fn test_execute() {
            let mut relayer = Relayer::new(10, AccountId::from([0; 32]), AccountId::from([0; 32]));
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
            let fee = 500000000u128;
            let refund = 500000000u128;
            let commitment = String::from(COMMITMENT);

            // NFT contract param
            let function = NFTFunction::Mint;
            let selector: [u8;4] = [0x18, 0x60, 0xff, 0x3b];
            let owner = AccountId::from([
                196, 250, 116, 227, 97, 67, 187, 105, 255, 166, 192, 240, 230, 161, 59, 203, 103, 129, 38,
                138, 170, 251, 216, 145, 117, 22, 187, 84, 152, 240, 21, 254,
            ]);

            const EPHEMERAL_PUBLIC_KEY: &str =
              "023283ba9bfc9f689cb4ca88d14734aea6e3bdded740d0e560e9344ab4fe825733";
            let ephemeral_public_key = EPHEMERAL_PUBLIC_KEY.to_string();
            let mut params: Vec<Param> = Vec::new();
            params.push(Param::AccountId(owner));
            params.push(Param::String(ephemeral_public_key));

            // Payable
            let accounts = default_accounts::<DefaultEnvironment>();
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            test::set_balance::<DefaultEnvironment>(accounts.alice, 10);
            test::set_value_transferred::<DefaultEnvironment>(1);

            relayer.deposit(commitment).unwrap();
            assert_eq!(
                relayer.execute(
                    proof,
                    root,
                    nullifier_hash.clone(),
                    recipient,
                    relayer_account,
                    fee,
                    refund,
                    function,
                    selector,
                    params
                ),
                Err(Error::WithdrawFailed)
            );
            assert_eq!(relayer.nullifier_hashes.contains(nullifier_hash), false);
        }

        #[ink::test]
        fn mimc_sponge() {
            // let inputs = vec![U256::ZERO.to_hex_string(), U256::ZERO.to_hex_string()];
            let inputs = vec![String::from(
                "471424a3bb441fde5e66071c0d74bac794d700cb8dbb8f1a996360870bc6ae",
            )];
            let relayer = Relayer::new(10, AccountId::from([0; 32]), AccountId::from([0; 32]));
            // println!("pow: {}", u64::pow(2, 10));
            relayer.mimc_sponge(inputs);
        }
    }
}

/// Returns a [`CallBuilder`] to a cross-contract call.
///
/// # Example
///
/// **Note:** The shown examples panic because there is currently no cross-calling
///           support in the off-chain testing environment. However, this code
///           should work fine in on-chain environments.
///
/// ## Example 1: No Return Value
///
/// The below example shows calling of a message of another contract that does
/// not return any value back to its caller. The called function:
///
/// - has a selector equal to `0xDEADBEEF`
/// - is provided with 5000 units of gas for its execution
/// - is provided with 10 units of transferred value for the contract instance
/// - receives the following arguments in order
///    1. an `i32` with value `42`
///    2. a `bool` with value `true`
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_call, Selector, ExecutionInput}
/// # };
/// # use ink_env::call::Call;
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// # type Balance = <DefaultEnvironment as Environment>::Balance;
/// let contract_id = AccountId::from([0x42; 32]);
/// let selector = [0xDE, 0xAD, 0xBE, 0xEF];
/// let args_1 = 42u8;
/// let args_2 = true;
/// crate::call!(contract_id, selector, args_1, args_2).returns::<()>().fire().unwrap();
/// ```
///
/// ## Example 2: With Return Value
///
/// The below example shows calling of a message of another contract that does
/// return a `i32` value back to its caller. The called function:
///
/// - has a selector equal to `0xDEADBEEF`
/// - is provided with 5000 units of gas for its execution
/// - is provided with 10 units of transferred value for the contract instance
/// - receives the following arguments in order
///    1. an `i32` with value `42`
///    2. a `bool` with value `true`
///    3. an array of 32 `u8` with value `0x10`
///
/// ```should_panic
/// # use ::ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// #     call::{build_call, Selector, ExecutionInput, Call},
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
/// let contract_id = AccountId::from([0x42; 32]);
/// let selector = [0xDE, 0xAD, 0xBE, 0xEF];
/// let args_1 = 42u8;
/// let args_2 = true;
/// let args_3 = &[0x10u8; 32];
/// let my_return_value: i32 = crate::call!(
///         contract_id,
///         selector,
///         args_1,
///         args_2,
///         args_3
///     )
///     .returns::<i32>()
///     .fire()
///     .unwrap();
/// ```
#[macro_export]
macro_rules! call {
        ( $contract:ident, $selector:ident, $( $arg:expr ),* ) => {
            {
                let args = ink_env::call::ExecutionInput::new(ink_env::call::Selector::new($selector));
                $(
                    let args = args.push_arg($arg);
                )*
                ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                    .call_type(
                        ink_env::call::Call::new()
                            .callee($contract)
                            .gas_limit(0)
                            .transferred_value(0),
                    )
                    .exec_input(args)
                    .call_flags(ink_env::CallFlags::default().set_allow_reentry(true))
            }
        };
    }
