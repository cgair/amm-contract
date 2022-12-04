# amm-contract
A simple AMM contract

## Prerequisites
To manually build this AMM contract, you will need to install the following dependencies:
* Rust
* Node.js
  
### Rust and Wasm
It is recommended to use [rustup](https://www.rust-lang.org/tools/install) to manage your Rust installation.
Then, add the wasm32-unknown-unknown toolchain which enables compiling Rust to Web Assembly (wasm), please execute the following command:
```bash
# Add the wasm toolchain
rustup target add wasm32-unknown-unknown
```

## Build
You just have to clone the repo and execute the following command:
```bash
cargo build  --target wasm32-unknown-unknown --release
```

## Deploy and Using
### NEAR Account system 
We deploy a contract, interact and query information from it through NEAR [Command Line Interface](https://github.com/near/near-cli) (CLI), which is a tool that enables to interact with the NEAR network directly from the shell.

In this example we need four accounts say jonhuang.testnet, a.jonhuang.testnet, b.jonhuang.testnet, z.jonhuang.testnet. jonhuang.testnet is the Master Account which I use to create subaccounts, deploy contract, manage contract. a.jonhuang.testnet, b.jonhuang.testnet and z.jonhuang.testnet are for A fungible token contract, B fungible token contract, AMM contract.

```bash
OWNER_ID=cgair.testnet  # <https://wallet.testnet.near.org/>
A_ID=a.cgair.testnet    # A fungible token contract 
B_ID=b.cgair.testnet    # B fungible token contract
AMM_ID=m.cgair.testnet  # AMM contract

near login
near create-account $A_ID --masterAccount $OWNER_ID
near create-account $B_ID --masterAccount $OWNER_ID
near create-account $AMM_ID --masterAccount $OWNER_ID
```

### Deploy Smart Contract
```bash
# near deploy $a_id --wasmFile="<path-to-contract>/token_contract.wasm"
# near deploy $b_id --wasmFile="<path-to-contract>/token_contract.wasm"
near deploy $amm_id --wasmFile="<path-to-contract>/amm_simple.wasm"
```



## Test
```bash
cargo test --all
```