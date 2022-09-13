# circuits

## Tools

[zokrates](https://zokrates.github.io/gettingstarted.html)

## Build circuits

run `build.sh` to compile the circuits and setup step to generate `proving.key` and `verification.key`

```sh
./build.sh
```

## ink! verifier contract

After built circuits

```sh
cd ../build/verifier
```

For new setup step, you need update the value of `VK` and `VK_GAMMA_ABC` from `verification.key` file.

```rust
#[ink::contract]
mod verifier {
    use ink_prelude::{string::String, vec, vec::Vec};
    use ink_storage::traits::SpreadAllocate;
    use zkmega_arkworks::{curve::Bn254, groth16};
    // VK = [alpha beta gamma delta]
    static VK: [&str; 14] = [];
    static VK_GAMMA_ABC: [&str; 70] = [];
    ....
}
```

**Warnning**: proving scheme only support groth16 for now.
