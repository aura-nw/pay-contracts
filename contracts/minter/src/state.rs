use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub receiver_name: String,
}

/// The information of exchanging
/// @param accepted_denom: The denom that the receiver want to receive.
/// @param address: The address of receiver's wallet. The new stable token will be minted to this address.
/// @param token_address: The address of stable token that the receiver want to receive.
/// @param price_feed: The address of price feed contract using to check exchange rate between stable token and native token.
#[cw_serde]
pub struct ExchangingInfo {
    pub accepted_denom: String,
    pub receiver_address: Addr,
    pub token_address: Addr,
    pub price_feed: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const EXCHANGING_INFO: Item<ExchangingInfo> = Item::new("exchanging_info");
