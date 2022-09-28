#![cfg_attr(not(feature = "std"), no_std)]

pub use self::verifier::{Verifier, VerifierRef};

use ink_lang as ink;
pub mod constants;

#[ink::contract]
mod verifier {
    use crate::constants::{VK, VK_GAMMA_ABC};
    use ink_prelude::{string::String, vec, vec::Vec};
    // use ink_storage::traits::SpreadAllocate;
    use zkmega_arkworks::{curve::Bn254, groth16};

    #[ink(storage)]
    // #[derive(SpreadAllocate)]
    pub struct Verifier {}

    impl Verifier {
        /// Use false as initial value
        #[ink(constructor)]
        pub fn default() -> Self {
            // ink_lang::utils::initialize_contract(|_| {})
            Self {}
        }

        #[ink(message)]
        pub fn verify(
            &self,
            proof: String,
            root: String,
            nullifier_hash: String,
            recipient: AccountId,
            relayer: AccountId,
            fee: u128,
            refund: u128,
        ) -> bool {
            let mut nullifier_vec = hex::decode(nullifier_hash).unwrap();
            nullifier_vec.reverse();
            let mut root_vec = hex::decode(root).unwrap();
            root_vec.reverse();
            // concat public inputs
            let inputs = ([
                root_vec,
                nullifier_vec,
                self.buff2input(recipient.as_ref()),
                self.buff2input(relayer.as_ref()),
                ([(fee).to_le_bytes(), [0u8; 16]]).concat(),
                ([(refund).to_le_bytes(), [0u8; 16]]).concat(),
            ])
            .concat();

            let proof_vec = hex::decode(proof).unwrap();
            let proof_and_input = ([proof_vec, inputs]).concat();
            groth16::preprocessed_verify_proof::<Bn254>(
                VK,
                VK_GAMMA_ABC.to_vec(),
                proof_and_input.as_slice(),
            ).unwrap()
        }

        pub fn buff2input(&self, buffer: &[u8]) -> Vec<u8> {
            let result: Vec<Vec<u8>> = buffer
                .chunks(16)
                .map(|m| {
                    let mut s = m.to_vec();
                    s.reverse();
                    ([s, vec![0u8; 16]]).concat()
                })
                .collect();
            result.concat()
        }
    }

    #[cfg(test)]
    mod tests {

        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_lang as ink;
        #[ink::test]
        fn test_verify() {
            let verifier = Verifier::default();
            let proof = String::from("d99fe3bfff6c0fb054febe6f484220320f1f12e32a7f12606731fdbc34dce81fb5b988a199ef9782a04fb6775a2ea82ccb002f483deca7ac63c2a8586d8cdc04005d4d217dd66ff559e986bb6b90a9f43b7dacd9c12d0d3342983b15c82ee7221a08cfd18355c38cc2cff8fc85d50915d960475dc4c1c07370dcf4a00a90b843223792df6a1e0f21b81a86e15db434e5a371e58f7818328355b3f235cf547afc2205985c8e6d0389a5b5b186c0541ad9187388e696d2fd97ddc394ba0908344929008470d0c05e20cbef4cb0b2baff948bbc5b9317d5a8524b20431082b7128d4202b535196cb47e02b79a94dd674ad38d8da9e64fe0b975ed662a2b3abb36ff572400");
            let root =
                String::from("222eddf0a52aada170d89dd492bf939c6430d4e10c0bf2b843e6bde7ac46781f");
            let nullifier_hash =
                String::from("15bd4d1ea3140c2a717b781050a6dd46f93a056f8a7e2f40cfd30740a2444a95");
            let recipient = AccountId::from([
                212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44,
                133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
            ]);
            let relayer = AccountId::from([
                142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97,
                54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72,
            ]);
            let fee = 1000000000u128;
            let refund = 2000000000u128;
            let result =
                verifier.verify(proof, root, nullifier_hash, recipient, relayer, fee, refund);
            assert!(result);
        }
    }
}
