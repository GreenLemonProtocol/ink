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
cd ../contract/verifier
```

For new setup step, you need update the value of `VK` and `VK_GAMMA_ABC` in `constants.rs` according to the `verification.key` file.

```rust
// VK = [alpha beta gamma delta]
pub static VK: [&str; 14] = [];
pub static VK_GAMMA_ABC: [&str; 18] = [];
```

**Note**: proving scheme only support groth16 for now.
