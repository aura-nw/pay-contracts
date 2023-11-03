use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

// the config of the price feed
#[cw_serde]
pub struct Config {
    pub owner: Addr,
}

#[cw_serde]
pub struct PriceFeedInfo {
    pub price_feed: Addr,
    pub latest_round: u64,
}

#[cw_serde]
pub enum RoundDataStatus {
    Pending,
    Answered,
    Rejected,
}

#[cw_serde]
pub struct Answer {
    pub provider: Addr,
    pub value: u64,
    pub updated_at_height: u64,
}

// the data struct of each round
#[cw_serde]
pub struct RoundData {
    pub started_at_height: u64,
    pub status: RoundDataStatus,
    pub answers: Vec<Answer>,
    pub answered_at_height: u64,
}

impl RoundData {
    pub fn is_answered(&self, provider: Addr) -> bool {
        self.answers.iter().any(|a| a.provider == provider)
    }

    pub fn current_answer(&self) -> u64 {
        // get the average of all answers
        let sum: u64 = self.answers.iter().map(|a| a.value).sum();
        if sum == 0 {
            0
        } else {
            sum / self.answers.len() as u64
        }
    }

    pub fn current_number_answeres(&self) -> usize {
        self.answers.len()
    }
}

// the config data
pub const CONFIG: Item<Config> = Item::new("config");
// the round data is stored in the map with the round id as the key
pub const ROUND_DATA: Map<u64, RoundData> = Map::new("round_data");
// the list of feeders
pub const FEEDERS: Map<Addr, bool> = Map::new("feeders");
// the price feed info
pub const PRICE_FEED_INFO: Item<PriceFeedInfo> = Item::new("price_feed_info");
