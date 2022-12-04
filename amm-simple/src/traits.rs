use near_sdk::{
    ext_contract, 
    AccountId, Balance,
    json_types::U128,
};

#[ext_contract(ext_token)]
trait ExtToken {
    /// Transfers positive amount of tokens from the env::predecessor_account_id to receiver_id. 
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128);
}

#[ext_contract(ext_c)]
trait ExtContract {
    /// Update reserves
    fn update(&mut self, reserve_a: Balance, reserve_b: Balance);

    /// Caculate K = X * Y
    fn calc_k(&mut self);
}