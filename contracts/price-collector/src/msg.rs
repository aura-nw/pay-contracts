use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::RoundData;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub controller: String,
    pub decimals: u8,
    pub description: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    UpdatePriceFeed { price_feed: String, status: bool },
    ProvideRoundData { answer: u64 },
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(RoundData)]
    LastestRoundData {},
    #[returns(RoundData)]
    RoundData { round_id: u64 },
}
