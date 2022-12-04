//! A simple AMM contract
use near_sdk::{
    env,
    borsh::{self, BorshSerialize, BorshDeserialize},
    near_bindgen,
    require,
    PanicOnDefault,
    AccountId, Balance,
};


mod traits;
use traits::*;

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
    reserve: Balance,
    /// the ticker symbol
    ticker: String,
    /// shifting all numbers by the declared number of zeros
    decimal: u8,
}

impl Token {
    pub fn new(account_id: AccountId, prefix: &str) -> Self {

        Self {
            account_id,
            reserve: 0,
            ticker: prefix.to_string(),
            decimal: 1,
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

        // Requests and stores the metadata of tokens (name, decimals)
        ext_token::ext(a_id.clone())
            .get_info()
            .then(
                ext_c::ext(env::current_account_id())
                .callback_get_info(a_id.clone()),
            );

        ext_token::ext(b_id.clone())
            .get_info()
            .then(
                ext_c::ext(env::current_account_id()).callback_get_info(b_id.clone()),
            );

        // Creates wallets for tokens А & В.
        ext_token::ext(a_id.clone()).register_amm(owner_id.clone(), 0);
        ext_token::ext(b_id.clone()).register_amm(owner_id.clone(), 0);

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
        (
            (self.token_a.account_id.clone(), self.token_a.ticker.clone(), self.token_a.reserve, self.token_a.decimal),
            (self.token_b.account_id.clone(), self.token_b.ticker.clone(), self.token_b.reserve, self.token_b.decimal)
        )
    }

    pub fn callback_get_info(&mut self, contract_id: AccountId, #[callback] val: (String, u8)) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Only support in self"
        );
        if contract_id == self.token_a.account_id {
            // self.a_contract_name = val.0;
            self.token_b.decimal = val.1;
        } else if contract_id == self.token_b.account_id {
            self.token_b.decimal = val.1;
        }
        self.calc_k();
    }

    /// Public - swaps tokens between the given account and the pool.
    pub fn swap(&mut self, token_in: AccountId, amount_in: Balance) {
        require!(token_in == self.token_a.account_id || token_in == self.token_b.account_id, "invalid token");
        require!(amount_in > 0, "amount in = 0");

        // Pull in token in;
        if token_in == self.token_a.account_id {
            // ext_token::ext()
            //     .with_attached_deposit(1)
            //     .ft_transfer(token_in, amount_in.into());
            let xx = self.token_a.reserve;
            let yy = self.token_b.reserve;

            let x = decimals(xx, self.token_a.decimal);
            let y = decimals(yy, self.token_b.decimal);

            // Calculate token out
            let amount_out = calc_dy(x, y, amount_in);
            
            let a_added = self.token_a.reserve + amount_in;
            let b_added = self.token_b.reserve - amount_out;

            // Transfer token out to sender
            ext_token::ext(env::predecessor_account_id())
            .ft_transfer(self.token_a.account_id.clone(), amount_in.into())
            .then(
                ext_c::ext(env::current_account_id())
                .callback_ft_deposit(
                    a_added,
                    b_added,
                    self.token_b.account_id.clone(),
                    env::predecessor_account_id(),
                    amount_out,
                ),
            );
        } else {
            let xx = self.token_b.reserve;
            let yy = self.token_a.reserve;

            let x = decimals(xx, self.token_b.decimal);
            let y = decimals(yy, self.token_a.decimal);

            // Calculate token out
            let amount_out = calc_dy(x, y, amount_in);
            
            let b_added = self.token_b.reserve + amount_in;
            let a_added = self.token_a.reserve - amount_out;

            // Transfer token out to sender
            ext_token::ext(env::predecessor_account_id())
            .ft_transfer(self.token_b.account_id.clone(), amount_in.into())
            .then(
                ext_c::ext(env::current_account_id())
                .callback_ft_deposit(
                    a_added,
                    b_added,
                    self.token_a.account_id.clone(),
                    env::predecessor_account_id(),
                    amount_out,
                ),
            );
        }
    }

    /// Add tokens to the liquidity pool.
    pub fn add_liquidity(&mut self, amount_a: Balance, amount_b: Balance) {
        require!(env::predecessor_account_id() == self.admin, "Must be owner");

        let a_added = self.token_a.reserve + amount_a;
        let b_added = self.token_b.reserve + amount_b;
        // Pull in token A and token B
        // TODO: Mint shares
        // Update reserves

        ext_token::ext(self.admin.clone())
            .ft_transfer(self.token_a.account_id.clone(), amount_a.into())
            .then(
                ext_c::ext(env::current_account_id())
                    .callback_update(a_added, b_added),
            );
        
        ext_token::ext(self.admin.clone())
            .ft_transfer(self.token_b.account_id.clone(), amount_b.into())
            .then(
                ext_c::ext(env::current_account_id())
                    .callback_update(a_added, b_added),
            );
    }

    // fn remove_liquidity()

    pub fn callback_update(&mut self, a_added: Balance, b_added: Balance) {
        self.token_a.reserve = a_added;
        self.token_b.reserve = b_added;
        self.calc_k();
    }

    pub fn callback_ft_deposit(
        &mut self,
        a_added: Balance,
        b_added: Balance,
        contract_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    ) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Only support to call by itself"
        );

        ext_token::ext(contract_id)
            .ft_transfer(receiver_id, amount.into())
            .then(
                ext_c::ext(env::current_account_id())
                    .callback_update(a_added, b_added),
            );
    }
    
    /// Caculate K = X * Y
    pub fn calc_k(&mut self) {
        let x = self.token_a.reserve / 10_u128.pow(self.token_a.decimal as u32);
        let y = self.token_b.reserve / 10_u128.pow(self.token_b.decimal as u32);
        
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
        // let contract_id = contract.as_account().id().clone();

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
        let _res0 = 
            contract.call("add_liquidity")
            // a_acount.call(&contract_id, "add_liquidity")
            .args_json(
                serde_json::json!({
                    "amount_a": 10,
                    "amount_b": 10
                })
            )
            .max_gas()
            .transact()
            .await?;

        let info0 = contract.
            call("get_info")
            .max_gas()
            .transact()
            .await?;
        println!("[+] info: {:?}", info0.json::<((AccountId, String, Balance, u8), (AccountId, String, Balance, u8))>()?);

        Ok(())
    }
}
