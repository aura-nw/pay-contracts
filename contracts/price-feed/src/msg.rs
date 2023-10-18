use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Timestamp, Uint128};

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
    UpdateController { controller: String },
    UpdateRoundData { answer: u64 },
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(RoundDataResponse)]
    LastestRoundData {},
    #[returns(RoundDataResponse)]
    RoundData { round_id: u64 },
    #[returns(u8)]
    Decimals {},
    #[returns(String)]
    Description {},
    #[returns(String)]
    Controller {},
}

// the data struct of each round
#[cw_serde]
pub struct RoundDataResponse {
    pub round_id: u64,
    pub answer: Uint128,
    pub updated_at: Timestamp,
    pub answered_in_round: u64,
}
