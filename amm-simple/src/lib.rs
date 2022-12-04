//! A simple AMM contract
use near_sdk::{
    env,
    borsh::{self, BorshSerialize, BorshDeserialize},
    json_types::U128,
    near_bindgen,
    require,
    PanicOnDefault,
    AccountId, Balance, Promise, log,
};

use near_contract_standards::{
    fungible_token::{
        FungibleToken,
        metadata::FungibleTokenMetadata,
    },
};


mod traits;
// use traits::*;

// *********************************************************
// * Data Structures for Simple AMM Contract
// * cgair 3 Dec 2022
// * https://docs.near.org/develop/contracts/whatisacontract
// *********************************************************

/// The [Contract Class](https://docs.near.org/develop/contracts/anatomy)
#[near_bindgen]
#[derive(PanicOnDefault, BorshSerialize, BorshDeserialize)]
pub struct Contract {
    /// contract owner
    admin: AccountId,
    /// xy = k
    k: u128,
    /// token A
    token_a: Token,
    /// token B
    token_b: Token,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshSerialize, BorshDeserialize)]
pub struct Token {
    account_id: AccountId,
    ft: FungibleToken,
    meta: FungibleTokenMetadata,
    // reserve: Balance,
    // /// the ticker symbol
    // ticker: String,
    // /// shifting all numbers by the declared number of zeros
    // decimal: u8,
}

impl Token {
    pub fn new(account_id: AccountId, prefix: &str) -> Self {
        let mut ft = FungibleToken::new(prefix.as_bytes());
        ft.internal_register_account(&account_id);

        Self {
            account_id,
            ft,
            meta: FungibleTokenMetadata { spec: "".to_string(), name: "".to_string(), symbol: prefix.to_string(), icon: None, reference: None, reference_hash: None, decimals: 1 }
        }
    }
}

fn init_token(account_id: AccountId, prefix: &str) -> Token {
    Token::new(account_id, prefix)
}

#[near_bindgen]
impl Contract {
    /// Public - initializing the State by storing the metadata(name, decimals) of tokens.
    #[init]
    pub fn init(owner_id: AccountId, a_id: AccountId, b_id: AccountId) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            admin: owner_id,
            k: 0,
            token_a: init_token(a_id, "a"),
            token_b: init_token(b_id, "b"),
        }
    }

    pub fn get_info(
        &self
    ) -> (
        (AccountId, String, Balance, u8),
        (AccountId, String, Balance, u8),
    )
    {
        let a_id = self.token_a.account_id.clone();
        let b_id = self.token_b.account_id.clone();
        (
            (self.token_a.account_id.clone(), self.token_a.meta.symbol.clone(), self.token_a.ft.internal_unwrap_balance_of(&a_id), self.token_a.meta.decimals),
            (self.token_b.account_id.clone(), self.token_b.meta.symbol.clone(), self.token_b.ft.internal_unwrap_balance_of(&b_id), self.token_b.meta.decimals)
        )
    }

    /// Public - swaps tokens between the given account and the pool.
    pub fn swap(&mut self, token_in: AccountId, amount_in: Balance) {
        require!(token_in == self.token_a.account_id || token_in == self.token_b.account_id, "invalid token");
        require!(amount_in > 0, "amount in = 0");

        let a_id = self.token_a.account_id.clone();
        let b_id = self.token_b.account_id.clone();
        let user_account_id = env::predecessor_account_id();

        // Pull in token in;
        if token_in == self.token_a.account_id {
            // ext_token::ext(env::predecessor_account_id())
            //     .with_attached_deposit(1)
            //     .ft_transfer(token_in, amount_in.into());

            let xx = self.token_a.ft.internal_unwrap_balance_of(&a_id);
            let yy = self.token_b.ft.internal_unwrap_balance_of(&b_id);

            self.token_a
                .ft
                .internal_transfer(&user_account_id, &a_id, amount_in, None);

            let x = decimals(xx, self.token_a.meta.decimals);
            let y = decimals(yy, self.token_b.meta.decimals);

            // Calculate token out
            let amount_out = calc_dy(x, y, amount_in);

            // Transfer token out to sender
            self.token_b
                .ft
                .internal_transfer(&b_id, &user_account_id, amount_out, None);
            /*
            ext_token::ext(self.token_b.account_id.clone())
                .with_attached_deposit(1)
                .ft_transfer(env::predecessor_account_id(), amount_out.into())
                .then(
                    // Update the reserve
                    ext_c::ext(self.admin.clone())
                    .update(xx + amount_in, yy - amount_out)
                );
            */
        } else {
            let xx = self.token_b.ft.internal_unwrap_balance_of(&b_id);
            let yy = self.token_a.ft.internal_unwrap_balance_of(&a_id);

            self.token_b
                .ft
                .internal_transfer(&user_account_id, &b_id, amount_in, None);

            let x = decimals(xx, self.token_b.meta.decimals);
            let y = decimals(yy, self.token_a.meta.decimals);

            // Calculate token out
            let amount_out = calc_dy(x, y, amount_in);

            // Transfer token out to sender
            self.token_a
                .ft
                .internal_transfer(&a_id, &user_account_id, amount_out, None);
        }
    }

    /// Add tokens to the liquidity pool.
    pub fn add_liquidity(&mut self, amount_a: Balance, amount_b: Balance) {
        let owner_id = env::predecessor_account_id();
        let a_id = self.token_a.account_id.clone();
        let b_id = self.token_b.account_id.clone();
        // Pull in token A and token B
        // TODO: Mint shares
        // Update reserves
        self.token_a
            .ft
            .internal_transfer(&owner_id, &a_id, amount_a, None);
        
        self.token_b
            .ft
            .internal_transfer(&owner_id, &b_id, amount_b, None);
        
        /*
        ext_token::ext(self.admin.clone())
        .with_attached_deposit(1)
        .ft_transfer(self.token_a.account_id.clone(), amount_a.into())
        .and(
            ext_token::ext(self.admin.clone())
            .with_attached_deposit(1)
            .ft_transfer(self.token_b.account_id.clone(), amount_b.into())
        )
        .then( // Update reserves
                ext_c::ext(self.admin.clone())
                .update(a_added, b_added)
            )
            .then(
                ext_c::ext(self.admin.clone())
                .calc_k()
            )
        */
    }

    // fn remove_liquidity()
    
    /// Caculate K = X * Y
    pub fn calc_k(&mut self) {
        let a_id = self.token_a.account_id.clone();
        let b_id = self.token_b.account_id.clone();

        let x = self.token_a.ft.internal_unwrap_balance_of(&a_id) / 10_u128.pow(self.token_a.meta.decimals as u32);
        let y = self.token_b.ft.internal_unwrap_balance_of(&b_id) / 10_u128.pow(self.token_b.meta.decimals as u32);
        
        self.k = x * y;
    }
}

// *********************************************************
// * Utils functions
// * cgair 2 Dec 2022
// *********************************************************
pub fn decimals(value: u128, decimals: u8) -> u128 {
    value * 10_u128.pow(decimals as u32)
}

/**
 * How much dy for dx?
 * 
 *  xy = k
 * (x + dx)(y - dy) = k
 * y - dy = k / (x + dx)
 * y - k / (x + dx) = dy
 * y - xy / (x + dx) = dy
 * (yx + ydx - xy) / (x + dx) = dy
 * ydx / (x + dx) = dy
*/
// TODO: fee 0.3%
pub fn calc_dy(x: u128, y: u128, dx: u128) -> u128 {
    y - (x * y / (x + dx))
}


// <https://docs.near.org/develop/testing/introduction>
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    // use near_sdk::{
    //     test_utils::{accounts, VMContextBuilder},
    //     testing_env,
    // };

    // fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
    //     let mut builder = VMContextBuilder::new();
    //     builder
    //         .current_account_id(accounts(0))
    //         .signer_account_id(predecessor_account_id.clone())
    //         .predecessor_account_id(predecessor_account_id);
    //     builder
    // }

    // #[test]
    // fn test_new() {
    //     let context = get_context(accounts(3));
    //     testing_env!(context.build());
    //     let contract = Contract::init(accounts(0), accounts(1), accounts(2));
    //     println!("{:?}", contract.get_info());
    // }
    use tokio::fs;
    // use near_sdk::test_utils::accounts;

    #[tokio::test]
    async fn workspaces_test() -> anyhow::Result<()> {
        let wasm = fs::read("res/amm_simple.wasm").await?;
        let worker = workspaces::sandbox().await?;

        let owner_account = worker.dev_create_account().await?;
        let owner_id = owner_account.id().clone();


        let (a_acount, b_acount) = (worker.dev_create_account().await?, worker.dev_create_account().await?);
        let (a_id, b_id) = (a_acount.id().clone(), b_acount.id().clone());

        let contract = worker.dev_deploy(&wasm).await?;
        let contract_id = contract.as_account().id().clone();

        // Call function a only to ensure it has correct behaviour
        let _ = 
            contract.call("init")
            // owner_account.call(&contract_id, "init")
            .args_json((owner_id,  a_id, b_id))
            // .args_json((contract_id, a_id, b_id))
            .max_gas()
            .transact()
            .await?;
        // test get_info
        let info0 = contract.
            call("get_info")
            .max_gas()
            .transact()
            .await?;
        println!("[+] info: {:?}", info0.json::<((AccountId, String, Balance, u8), (AccountId, String, Balance, u8))>()?);
 
        // test add_liquidity
        let res0 = 
            contract.call("add_liquidity")
            // owner_account.call(&contract_id, "add_liquidity")
            .args_json(
                serde_json::json!({
                    "amount_a": 10,
                    "amount_b": 10
                })
            )
            .max_gas()
            .transact()
            .await?;
        println!("{:?}", res0);

        let info0 = contract.
            call("get_info")
            .max_gas()
            .transact()
            .await?;
        println!("[+] info: {:?}", info0.json::<((AccountId, String, Balance, u8), (AccountId, String, Balance, u8))>()?);

        Ok(())
    }
}
