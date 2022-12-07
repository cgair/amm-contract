use near_sdk::{
    ext_contract, 
    AccountId, Balance,
};

#[ext_contract(ext_token)]
trait ExtToken {
    fn get_info(&self) -> (String, u8);
    fn register_amm(&mut self, sender_id: AccountId, amount: Balance);
    /// Transfers positive amount of tokens from the sender_id to receiver_id. 
    fn transfer_from(&mut self, sender_id: AccountId, receiver_id: AccountId, amount: Balance);
}

#[ext_contract(ext_c)]
trait ExtContract {
    fn callback_ft_deposit(
        &mut self,
        a_ticker_after: Balance,
        b_ticker_after: Balance,
        contract_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    );
    fn callback_get_info(&mut self, contract_id: AccountId, #[callback] val: (String, u8));
    /// Update reserves callback
    fn callback_update(&mut self, a_added: Balance, b_added: Balance);
    /// Caculate K = X * Y
    fn calc_k(&mut self);
}