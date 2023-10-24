use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;

/// Message type for `instantiate` entry_point
/// Maybe we don't need a new cw20 contract, just use the cw20-base contract
#[cw_serde]
pub struct InstantiateMsg {
    pub receiver_name: String,
    pub receiver_address: String,
    pub accepted_denom: String,
    pub price_feed: String,
    pub token_code_id: u64,
    pub token_instantiation_msg: Cw20InstantiateMsg,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    Exchange {
        amount: Uint128,
        expected_received: Uint128,
    },
    Withdraw {},
}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    Owner {},
    #[returns(ReceiverResponse)]
    Receiver {},
    #[returns(ExchangingInfoResponse)]
    ExchangingInfo {},
}

#[cw_serde]
pub struct ReceiverResponse {
    pub name: String,
    pub address: String,
}

#[cw_serde]
pub struct ExchangingInfoResponse {
    pub accepted_denom: String,
    pub token_address: String,
    pub price_feed: String,
}
