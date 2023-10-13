use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::RoundData;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    UpdateRoundData { answer: u64 },
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
}

// We define a custom struct for each query response
// #[cw_serde]
// pub struct YourQueryResponse {}
