use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

// the config of the price feed
#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub controller: Addr,
}

// information of the price feed
#[cw_serde]
pub struct PriceFeedInfo {
    pub latest_round: u64,
    pub decimals: u8,
    pub description: String,
    pub version: u64,
}

// the data struct of each round
#[cw_serde]
pub struct RoundData {
    pub answer: Uint128,
    pub started_at: Expiration,
    pub updated_at: Expiration,
    pub answered_in_round: Uint128,
}

// the config data
pub const CONFIG: Item<Config> = Item::new("config");
// the price feed info data
pub const PRICE_FEED_INFO: Item<PriceFeedInfo> = Item::new("price_feed_info");
// the round data is stored in the map with the round id as the key
pub const ROUND_DATA: Map<u64, RoundData> = Map::new("round_data");
