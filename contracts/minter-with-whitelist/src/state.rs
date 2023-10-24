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

#[cw_serde]
pub enum AssetType {
    NativeToken { denom: String },
    CW20Token { cw20_address: String },
    CW721Token { cw721_address: String },
}

#[cw_serde]
pub struct Asset {
    pub asset_type: AssetType,
    pub amount: u128,
}

#[cw_serde]
pub enum Requirement {
    All {},
    Any { at_least: u32 },
}

#[cw_serde]
pub struct RequirementAssets {
    pub assets: Vec<Asset>,
    pub required: Requirement,
}

impl RequirementAssets {
    pub fn update_asset(&mut self, asset: Asset) {
        // check if the asset is already in the list
        for a in self.assets.iter_mut() {
            match (&a.asset_type, &asset.asset_type) {
                (
                    AssetType::NativeToken { denom },
                    AssetType::NativeToken { denom: other_denom },
                ) => {
                    if denom == other_denom {
                        a.amount = asset.amount;
                        return;
                    }
                }
                (
                    AssetType::CW20Token { cw20_address },
                    AssetType::CW20Token {
                        cw20_address: other_cw20_address,
                    },
                ) => {
                    if cw20_address == other_cw20_address {
                        a.amount = asset.amount;
                        return;
                    }
                }
                (
                    AssetType::CW721Token { cw721_address },
                    AssetType::CW721Token {
                        cw721_address: other_cw721_address,
                    },
                ) => {
                    if cw721_address == other_cw721_address {
                        a.amount = asset.amount;
                        return;
                    }
                }
                _ => {}
            }
        }
        self.assets.push(asset);
    }

    pub fn remove_asset(&mut self, asset_type: AssetType) {
        self.assets.retain(|a| match (&a.asset_type, &asset_type) {
            (AssetType::NativeToken { denom }, AssetType::NativeToken { denom: other_denom }) => {
                denom != other_denom
            }
            (
                AssetType::CW20Token { cw20_address },
                AssetType::CW20Token {
                    cw20_address: other_cw20_address,
                },
            ) => cw20_address != other_cw20_address,
            (
                AssetType::CW721Token { cw721_address },
                AssetType::CW721Token {
                    cw721_address: other_cw721_address,
                },
            ) => cw721_address != other_cw721_address,
            _ => true,
        });
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const EXCHANGING_INFO: Item<ExchangingInfo> = Item::new("exchanging_info");
pub const REQUIREMENT_ASSETS: Item<RequirementAssets> = Item::new("requirement_assets");
