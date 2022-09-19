#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

pub mod generators;
pub mod jubjub;

#[ink::contract]
mod anonymous {
    use crate::generators::{Generators, SCALAR_FIELD};
    use crate::jubjub::{scalar_add, scalar_mult};
    use bitvec::field::BitField;
    use bitvec::prelude::{bitvec, BitVec, Msb0};
    use ink_prelude::string::ToString;
    use ink_prelude::{string::String, vec, vec::Vec};
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use scale::{Decode, Encode};
    use zkmega_arkworks::{curve::Bn254, groth16};
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
        BadLength,
    }
    // VK = [alpha beta gamma delta]
    static VK: [&str; 14] = [
        "0x142ccb07868608a95fdd8a30bc139fd759fb24882b3f15b292ec37303f65e348",
        "0x2a482afe143ccb036d74e3798f1cd8a1af53fdf29763edc7095a7105909188d1",
        "0x2167d2221e6e56a83823a461d056c810d1e27fd2dec9be2a0317e55d37a73cf0",
        "0x0896e91111c267a07fccb10d287430efe29140072a3a59fc3becd95b4bb7ad5c",
        "0x023a6cd28d6cd34929a1438d59745c19d942751fbc6d6bd8243eb7b835f48dc5",
        "0x209758438c285907cb31308868a5d1b42c28dc0650dc78b2d1ce9c357a604459",
        "0x151f1a146a399863f5008510d264314e5aaddf8d17bed2d66d51451c18670f81",
        "0x2962621db15aa88ce0c80e023cdec315b3019ae12c4af16f329bd5186e55f557",
        "0x116c515bb2908591feac79606ec3e3887e9364eb7820155a71447249f0dcbd46",
        "0x1276affffc073502b0b53c5f1718f1640dfe3301ef2d5f9703504a4e6054b7a6",
        "0x12e6b60a392d09f727431249a8f551b27422f9bc8977d613dde18715f5e1e755",
        "0x1abb3968e2096bae04f69cb12efcf49a02753a8f41da8c4e8c109b10439aed95",
        "0x1194e9e9a8a9b428ca9a068410d5233dfb30f59ac40db841c84d106070dabf72",
        "0x16095c71cb39c7b11f8bb30bc56fd9c28f8ab500de9c5656ee7cb75083631400",
    ];
    static VK_GAMMA_ABC: [&str; 70] = [
        "0x226364acca7b18c49440b1dfeda1bd91da3794b2e870f5a249535eca16962d79",
        "0x15ea05afd3dc915faa1659b448fddd605f44e7427ace6e5730ec40f69e351554",
        "0x2973e99251d4594b92309606dbfa2ca842cb9e9cd6075816328178e3d7f3c309",
        "0x08c24849261be5ae9a026bb66dacebd67615a5c98e1123825ea4e98322be292e",
        "0x2ae8bd9f6d8ea3baf1ce225b84c686c001bc4fba8e0be165985f03668b9920b1",
        "0x0d08d08233238de401df513a6b4d282a3e4fb5c488581c8185134f8dd60a550f",
        "0x1f395f6faed2d6272ba986e5e80afb7308930f338280fc5c964d50795e232e6b",
        "0x2626e9b677945deedeb58b34ca096ef022025f80f6e74bb1711b9ad064872d96",
        "0x00a9d8099d0e398abdc4c51a40872d73d6d65dbcd65c00c26b4b95fadb5b9c27",
        "0x065602c7eb651dd8c206ae82d0312ff0675ef097414c7ff66359627ebadf4d7e",
        "0x1951d80e15a188a0a028445ec11206e160e9b9c7e2c86f2bf1ddc1c2d36f096f",
        "0x1c08a7523f2d4816c85c43f24646c6cd87a3292cb3cb8723e1524399b96e3762",
        "0x28fde26d1a9fcbfd464db381ff3a4afbb302ef52d3721512df26e4a0fd4bb6f8",
        "0x2030079313d02ddc7d8f1975e8488a01d1d1db748cd2949884ae06b0dab1da1f",
        "0x09b487e6b62bbc9538c4e0a236fbd381786c12e20c897eeec73cf1464d852f82",
        "0x174900691a9ac078058afc18c8950920395e72c2e44886a57060c92cc822b5f5",
        "0x1cdf93df59def3655e65a488c3ce2ddd72a5a5385c3fb2200d9bf85ee5aa1b0a",
        "0x19d61cf00c30d131432b1f0d88b62fa5375497ccf962ac16945c2d745e477562",
        "0x2eb4b40c2e60c6aaea14d3248ceba47186f8e5c9fd286f24d55fef53cbf79c2f",
        "0x05d313b3493d9bba8c08659160ea65f8f1a801acf174d256561c8ea5f69dec5e",
        "0x03d3705b3e62241de666d7b9089de9eca2a8a0fd1dab2923da16a5e42dff54b2",
        "0x1012263eb79a1da32dc360d24c5cfa5ecc4b0d24684d50cf5bb89943b077cf0b",
        "0x185ed3908c2bd806c315b646c6c26a3529c8109c579f4e4a328d87452282f488",
        "0x29f324ed73cd5f7b59293f7fa43312705edfe7f9a409156ae4bad84c9f5d3408",
        "0x0ca077ad91ee6fcf8c9887ef35767ecde963f515d6890b4ceb8ffef36cc3fe01",
        "0x23a47d81191061e0e0dd7fac4667feb16725bc8f54470aeef73aadabb92c3fcd",
        "0x097da97d5c1cdf42951b396aabe8e45681c158811b4053b1625df33331b38b88",
        "0x059c58c8b73d0bce60e844bfbade29275498ab26de2c48cc92aeb9f34ec4a45d",
        "0x2605b63374fa1eea724ade682f32ffbd2c29e4f530a76640ce305e80fad6064a",
        "0x02778b350b198354a2e3a5bfc4be4399a2639cb53b113c18c0d73db029c289d7",
        "0x13b6dcfc9a2f333fb6b05fc880efcf3dac5681837dba8f38efd766ccc9f485c3",
        "0x17767ba3b1589562504c2f6e82b2e06867bad7e2704f40bd7d716349f6cc8856",
        "0x1e456dc255ce4f16b872e1d607c3a748900c53b699c3efd2f924635618b8c97a",
        "0x12e59250fa6a615b3d0e5a9abcb6be7bcae2b05890fb347737d6455786fed1d2",
        "0x25da7034839e0a7671862416eb12f985608e897985928fb14ed90349be2ffcf4",
        "0x2ff0c326e28e88662be00557a77138c88690ce21de33735bec093012407d8173",
        "0x193b32b07cea0aa63695684b3c0e4ff3860c66f27592da59d641e8967e3bcb6b",
        "0x26f330b6b7a307b57f06e3f1d4cbdf36d117e95af788ba54d281ec74aec584a2",
        "0x18edf72b4309201d315961f42dcaa55c77edbf9520f269dffaafe2ae4fb6b78e",
        "0x10fd650d3137a6d74150444cee8a0bcdf948819d9220b7526c963c24dcea0b50",
        "0x1e4f0254252b4988f1fdc3b66f1b75a439bb174672103797df279413ee3382ca",
        "0x051391bee5257d6875af65a41a88ec47f6884bff833974fa2824bd9d5c4d9d48",
        "0x063f34e24c3e0d4e690d4a788cd06f10e1d9379a52b13a38235d17d008f1d24a",
        "0x17cd41ddefba46597792337f46dd8a3e01adb48c4807458be970d8a54fbb69f1",
        "0x04c11eae2520cd58ee805d8a7f2781ceadc31259e6a29f6b78329dfa1b56afb0",
        "0x07d02f657a4306ddef1321fc494e79d0e34c482b52d58b94b42ce3104fb341fc",
        "0x038573e7d6cd55faf896ca52b1ed63bc2667e6d76589490e3a31de4480f1afc8",
        "0x12edcbddccfbf2a67b1cb3990f108ca4040dca3fed0820bda6cca16ff9b7e4f1",
        "0x1e90b1d2525dca3caf1d0c8f215f7dd23f8e745c219db123ffcf400dcb6bc747",
        "0x2e89df6d2f3d974287567dbe49a672a031c1ecca5661ca893abbc63d5c9231eb",
        "0x0ff04515113b0c36ae372a7b297bead07c7e60333eba69a146a8b2cc61e1576b",
        "0x11822b016f2fcc1ccb6d4ac279905b0add6a2b0628ae2c8c82eeb6e97df3482f",
        "0x194b0acf8ff1d7ad17dc5489f058e1396102edd7a0b1136b7cc76a51f5f18a58",
        "0x15545a18edd2c3464728f18bc9cc0c76c2b150fd99afc033045b4a1f2b782d88",
        "0x0db514d6a30766757dd95876de0ed8a636072cfdd2254f346302ec5696dead56",
        "0x13f0e5ecbb1daab520119394325a638c1f90cb9bbed4069a55478c9c1b2cb8bb",
        "0x2acb6bf47e041af0b361f204ac6ddbaf396a7d4a00313aab2d9f55dfbec42874",
        "0x00da40d170afb0c2914af671ac5d1bdcfe014e2ddc1b5d717f01ec298cb08c6e",
        "0x1c0bf7fdecfb71f4626a23ff2617cf532bf85fd07c911c4ccc5cf70b6acedd0f",
        "0x12b4a7db5c9919a9067b118b21def210e20269d22ddc0a81c6240ff435db2d54",
        "0x26be946706dd5f4d0938c6254a222c9a1a9a1384f6a47243abba8e4ace8ca861",
        "0x1d5b19cbe16eb296d66cd73407e8ea05a06f7919a108f9557147e2a243b698ab",
        "0x2869fa128ae8cb5d73662f288f4c5fe54231249544bc058aeaab3234290b4bd7",
        "0x23eff4d9a12ebb8183897f318fe2fffba05eb403cc493e4ef9996d0439774b9d",
        "0x01ec846228d8a59a8e3e27b3e15c549c0dd3b3f8afb89460b132df5ded19215d",
        "0x243168b807cb30d7e8fb35e9cb70765609598c6e54beea60caaff1806c37e5d5",
        "0x133eccdf30f4bbfea2447915cfecbe37b4cbba9479ad5b3995ccde52b88dd26b",
        "0x1b30e20901bedfc5cb9a6abf7faf2eefa0043d27bd1b80364b57129c13543324",
        "0x24eca9492c31e1d5fc72f0fa51df64aff4df8b72cbff9eefa3e5da0b549b9576",
        "0x127f38dfefb68afb25a0803e159f0cfa4faca47cdff73b7f21ba8db6879d8a82",
    ];

    const ROOT_HISTORY_SIZE: u32 = 30;
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Anonymous {
        // Stores the ZK result
        pub commitments: Mapping<String, bool>,
        pub nullifier_hashes: Mapping<String, bool>,
        pub levels: u32,
        pub filled_subtrees: Mapping<u32, String>,
        pub roots: Mapping<u32, String>,
        pub current_root_index: u32,
        pub next_index: u32,
    }

    impl Anonymous {
        #[ink(constructor)]
        pub fn new(levels: u32) -> Self {
            ink_lang::utils::initialize_contract(|contract| {
                Self::new_init(contract, levels);
            })
        }

        fn new_init(&mut self, levels: u32) {
            self.levels = levels;
            for i in 0..levels {
                self.filled_subtrees.insert(i, &self.zeros(i).unwrap());
            }
            self.roots.insert(0, &self.zeros(levels - 1).unwrap());
        }

        #[ink(message)]
        pub fn deposit(&mut self, commitment: String) -> Result<u32, Error> {
            if self.commitments.contains(commitment.clone()) {
                return Err(Error::AlreadySubmitted);
            }
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
            if !self.verify(
                &proof,
                &root,
                &nullifier_hash,
                recipient,
                relayer,
                fee,
                refund,
            )? {
                return Err(Error::InvalidWithdrawProof);
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

        // #[ink(message)]
        pub fn verify(
            &self,
            proof: &String,
            root: &String,
            nullifier_hash: &String,
            recipient: AccountId,
            relayer: AccountId,
            fee: u128,
            refund: u128,
        ) -> Result<bool, Error> {
            // concat public inputs
            let inputs = ([
                self.buff2input(hex::decode(root).unwrap().as_slice())?,
                self.buff2input(hex::decode(nullifier_hash).unwrap().as_slice())?,
                self.buff2input(recipient.as_ref())?,
                self.buff2input(relayer.as_ref())?,
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
            )
            .map_err(|_| Error::VerifyCatchErr)
        }

        pub fn buff2input(&self, buffer: &[u8]) -> Result<Vec<u8>, Error> {
            if buffer.len() != 32 {
                return Err(Error::BadLength);
            }
            let result: Vec<Vec<u8>> = buffer
                .chunks(4)
                .map(|m| {
                    let mut s = m.to_vec();
                    s.reverse();
                    ([s, vec![0u8; 28]]).concat()
                })
                .collect();
            Ok(result.concat())
        }

        // pederson hash
        pub fn hash_left_right(&self, left: String, right: String) -> String {
            let preimage = left + &right;
            // ink_env::debug_println!("preimage: {}", preimage);
            let decode = hex::decode(preimage).unwrap();
            let bits = BitVec::<_, Msb0>::from_vec(decode);
            let windows: Vec<u8> = bits
                .chunks(3)
                .map(|a| {
                    let mut b = BitVec::<_, Msb0>::from(a);
                    if b.len() < 3 {
                        b.append(&mut bitvec![0; 3 - b.len()]);
                    }
                    b.reverse();
                    b.load_be::<u8>()
                })
                .collect();
            let generators = Generators::get_generators();
            let mut result: (U256, U256) = (U256::ZERO, U256::ONE);
            let scalar_field = U256::from_decimal_str(SCALAR_FIELD).unwrap();
            for (g, window) in generators.iter().zip(windows.iter()) {
                let value =
                    U256::from_decimal_str(((*window & 0b11) + 1).to_string().as_str()).unwrap();
                // ink_env::debug_println!("value {:?}", value);
                let mut segment = scalar_mult(
                    U256::from_decimal_str(g.0).unwrap(),
                    U256::from_decimal_str(g.1).unwrap(),
                    value,
                )
                .unwrap();
                // ink_env::debug_println!("{}, {}", segment.0.to_decimal_string(), segment.1.to_decimal_string());
                if *window > 0b11 {
                    segment.0 = scalar_field.clone() - segment.0;
                }
                result = scalar_add(result, segment).unwrap();
            }
            // compress
            let compress = result.1 | ((result.0 & U256::ONE) << 255);
            hex::encode(compress.to_bytes_be())
        }

        #[ink(message)]
        pub fn insert(&mut self, leaf: String) -> Result<u32, Error> {
            // self.next_index = nextndex;
            let next_index = self.next_index;
            if next_index >= 2 ^ self.levels {
                return Err(Error::MerkleTreeFull);
            }
            let mut current_index = next_index;
            let mut current_level_hash = leaf;
            let mut left: String;
            let mut right: String;
            for i in 0..self.levels {
                if current_index % 2 == 0 {
                    left = current_level_hash.clone();
                    right = self.zeros(i)?;
                    self.filled_subtrees.insert(i, &current_level_hash);
                } else {
                    left = self.filled_subtrees.get(i).unwrap();
                    right = current_level_hash.clone();
                }
                current_level_hash = self.hash_left_right(left, right);
                current_index /= 2;
            }
            let new_root_index = (self.current_root_index + 1) % ROOT_HISTORY_SIZE;
            self.current_root_index = new_root_index;
            self.roots.insert(new_root_index, &current_level_hash);
            self.next_index = next_index + 1;
            Ok(next_index)
        }

        pub fn zeros(&self, i: u32) -> Result<String, Error> {
            if i == 0 {
                Ok(String::from(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                ))
            } else if i == 1 {
                Ok(String::from(
                    "a65f3fa0002aba81ad5f5805158ca53b4c6786ad9dc9845a0acbd5e718ffe95d",
                ))
            } else if i == 2 {
                Ok(String::from(
                    "ada59a6d0ece673d3f6bf770b0602c7e15b3c154828fe67ee6e1c54250145956",
                ))
            } else if i == 3 {
                Ok(String::from(
                    "09683c3018a3309dc76040963de2179757aacd7fc166a21493603436c396fbf0",
                ))
            } else if i == 4 {
                Ok(String::from(
                    "84de6a44dbdf9e40df97ef79992bdde8a31f6d29cd8a59f89c5070c3a4065f33",
                ))
            } else if i == 5 {
                Ok(String::from(
                    "04638c0dcdf5c52f749a5fdcdf5241dc5033e3af46602de9075cb6023f5fcdba",
                ))
            } else if i == 6 {
                Ok(String::from(
                    "2aa147eff6a8e373c78805bf685e0637e26e264c62ae9b92c810e439eaf00317",
                ))
            } else if i == 7 {
                Ok(String::from(
                    "8a3760aea515435ac5b92c6f959a403ccabae2f4a9d4cf3e06e27b89ef2a1e34",
                ))
            } else if i == 8 {
                Ok(String::from(
                    "87d05dbd193f385bf2b08418b5b36690057f47f89fec12cdd1ce9113de31e5f7",
                ))
            } else if i == 9 {
                Ok(String::from(
                    "996f1623b254c140d3c609c6a861d6fe40f84158d4ec47ccdf7705d7f78fee02",
                ))
            } else if i == 10 {
                Ok(String::from(
                    "22f78ae35c2b9178ea4c0c4167de764855502fbc1b06b1f459b0bc39f6e0f601",
                ))
            } else if i == 11 {
                Ok(String::from(
                    "a544828f0c4c3f2b482c2a034cee8b06660c2c5ad58441ff4163568f45b43312",
                ))
            } else if i == 12 {
                Ok(String::from(
                    "103c9689358eb534426ed9a16c2b38512ae198a96ef5c6f8ada11089a36a6d29",
                ))
            } else if i == 13 {
                Ok(String::from(
                    "0049295c7dd9680c0e6e7c69bc55bd568ebdf36027145653763a503e79728dd4",
                ))
            } else if i == 14 {
                Ok(String::from(
                    "82554149ee13fc4c6047cfee1c48554bc9014dd3bc6574ce3023da1961db2641",
                ))
            } else if i == 15 {
                Ok(String::from(
                    "08f6c589709c5622413d276c3b180d60d43dbff9ee9584fd14a05f5072bf96ef",
                ))
            } else if i == 16 {
                Ok(String::from(
                    "8ba835bde5335cde7ae441ccbdba1b691c161a3ab701b8e3f607855e716864e5",
                ))
            } else if i == 17 {
                Ok(String::from(
                    "1313cda6dd1d4c5b25c8df67098de0972ee30e44537526386502b7cbe980ba6d",
                ))
            } else if i == 18 {
                Ok(String::from(
                    "a0daff11b366059a5298b7fda75f8bcacb03398818e985f44cc9135761371285",
                ))
            } else if i == 19 {
                Ok(String::from(
                    "921f29bd173979e2f9f4b8625c5d1a24f3443ee8935b1eef02f07de494c20a24",
                ))
            } else if i == 20 {
                Ok(String::from(
                    "82b9504d0be1e0b9b12f95db5acd952ad5326139de16350a92117e93552eb054",
                ))
            } else if i == 21 {
                Ok(String::from(
                    "96b164d69cb7978ce59ee00907a47ec1a5f7242644149ec15a0b0321b526045a",
                ))
            } else if i == 22 {
                Ok(String::from(
                    "94faf3131f84cba4b302240dc8b4a1b92d949272faf01e00356eac8dd49b6558",
                ))
            } else if i == 23 {
                Ok(String::from(
                    "2c0e9b39a5a6aabe7321e035d7c4ee2e5fa453bb0181be99d52d6468d9b4f82f",
                ))
            } else if i == 24 {
                Ok(String::from(
                    "8049c5b65bc12ec0d045986ba2d1c16eaef21628303143499dfafd515841b948",
                ))
            } else if i == 25 {
                Ok(String::from(
                    "131bca1bd4956ecd6a77bf4e79ab4e545504accd27bdf39975e2fa1e14eec436",
                ))
            } else if i == 26 {
                Ok(String::from(
                    "1e0d8ec73d32e7c6e0bd288c36a1632a34a97ce755ea8a517071c672325826a4",
                ))
            } else if i == 27 {
                Ok(String::from(
                    "83a22796183cfcf5fd119571ae5b596e4d776e2881e55e7ce7d10120e3e41e11",
                ))
            } else if i == 28 {
                Ok(String::from(
                    "850c8ff3afbe9fb7cf39206eecdb4f7d9ac45ce7a71cc3ccea8ee3d8f3c2d427",
                ))
            } else if i == 29 {
                Ok(String::from(
                    "221e8f4d5ca8862c7ce818ac0b02b4929ce36764aaf2576e7ae4de17111045f2",
                ))
            } else if i == 30 {
                Ok(String::from(
                    "15373dc7c73eaa97ad0e6a42351d6bd536ca37822c012cc7648498adb2e830d9",
                ))
            } else if i == 31 {
                Ok(String::from(
                    "a2a8d8d4af9d552790e9c28b852640593bebd4727cf9450b69e2166ecefe1820",
                ))
            } else if i == 32 {
                Ok(String::from(
                    "2dd5c52f3b2e5194699ac32d7a37db3c418768161f26b1ea06639afefa0dda9e",
                ))
            } else {
                Err(Error::IndexOutOfBounds)
            }
        }
    }

    #[cfg(test)]
    mod tests {

        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_env::{
            self,
            test::{self, default_accounts},
            DefaultEnvironment,
        };
        use ink_lang as ink;
        const ROOT: &str = "9faf1f92296668ca4c56a22cbbd20f70a65a49cf35aecc28df30e1c371fef7b9";
        const COMMITMENT: &str = "8dd43ca82ebd13c6cb851c74f07e0dc753643b9a522b6d09529ed3456fe3dacc";
        const NULLIFIER_HASH: &str =
            "2da3e23f162a789fdf0f0dd2f6d1b8fa7fcb4cd54f8d1afba878081ebcf49621";
        const PROOF: &str = "720111047847c3da8dae5b0b85d9b9ea46694889f7ad2d4f8c7a669dcc902a064ba8923460f1afaf4aa75b4ee69ab92c73c5e9362de0f90a724b10a103e2df1c006405caa7ea259d9c643082fb4ff140f01a1e6b20e8f3f9859e5e9ec92d0919048b910cf3828ed92ad1276e6828106e4e21f43722575aaaa8c69401708cdbd71bc1500fb01d2ba3cf302223fb1ab5ef62812828cfc287a2cb6ca92082b0aaea188c5fdf9262acad9d3aa4871f6a617125ea3582e12fe2c9c0bb0058171e34011500e84f0cb4e30577ea956e97f00258fbdfbc636da75b34b9e585864ad09b70e127ec81aab10822591688cc3f3e2c52f41b3b1ad7a52b8ca586ccb57b89978ec00000";

        #[ink::test]
        fn test_deposit() {
            let accounts = default_accounts::<DefaultEnvironment>();
            let mut anonymous = Anonymous::new(10);
            test::set_caller::<DefaultEnvironment>(accounts.alice);
            let commitment = String::from(COMMITMENT);
            let root = String::from(ROOT);
            anonymous.deposit(commitment).unwrap();
            assert!(anonymous.is_known_root(root));
        }

        #[ink::test]
        fn test_withdraw() {
            let mut anonymous = Anonymous::new(10);
            let proof: String = String::from(PROOF);
            let root: String = String::from(ROOT);
            let nullifier_hash: String = String::from(NULLIFIER_HASH);
            // the recipient SS58Address is "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            let recipient = AccountId::from([
                212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44,
                133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
            ]);

            // the recipient SS58Address is "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
            let relayer = AccountId::from([
                142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97,
                54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72,
            ]);
            let fee = 1000000000u128;
            let refund = 2000000000u128;
            let commitment = String::from(COMMITMENT);
            anonymous.deposit(commitment).unwrap();
            anonymous
                .withdraw(proof, root, nullifier_hash.clone(), recipient, relayer, fee, refund)
                .unwrap();
            assert!(anonymous.nullifier_hashes.get(nullifier_hash).unwrap());
        }
    }
}
