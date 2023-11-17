use cosmwasm_schema::cw_serde;

use cosmwasm_std::{StdError, StdResult, Uint128};
use cw20::{Cw20Coin, MinterResponse};
use cw_storage_plus::Item;

/// TokenContract InstantiateMsg
#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<Cw20Coin>,
    pub mint: Option<MinterResponse>,
    pub native_denom: String,
}

impl InstantiateMsg {
    pub fn get_cap(&self) -> Option<Uint128> {
        self.mint.as_ref().and_then(|v| v.cap)
    }

    pub fn validate(&self) -> StdResult<()> {
        // Check name, symbol, decimals
        if !self.has_valid_name() {
            return Err(StdError::generic_err(
                "Name is not in the expected format (3-50 UTF-8 bytes)",
            ));
        }
        if !self.has_valid_symbol() {
            return Err(StdError::generic_err(
                "Ticker symbol is not in expected format [a-zA-Z\\-]{3,12}",
            ));
        }
        if self.decimals > 18 {
            return Err(StdError::generic_err("Decimals must not exceed 18"));
        }
        Ok(())
    }

    fn has_valid_name(&self) -> bool {
        let bytes = self.name.as_bytes();
        if bytes.len() < 3 || bytes.len() > 50 {
            return false;
        }
        true
    }

    fn has_valid_symbol(&self) -> bool {
        let bytes = self.symbol.as_bytes();
        if bytes.len() < 3 || bytes.len() > 12 {
            return false;
        }
        for byte in bytes.iter() {
            if (*byte != 45) && (*byte < 65 || *byte > 90) && (*byte < 97 || *byte > 122) {
                return false;
            }
        }
        true
    }
}

#[cw_serde]
pub struct SupportedNative {
    pub denom: String,
}

pub const SUPPORTED_NATIVE: Item<SupportedNative> = Item::new("supported_native");

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_cap() {
        let msg = InstantiateMsg {
            decimals: 6u8,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                cap: Some(Uint128::from(1u128)),
                minter: "minter0000".to_string(),
            }),
            name: "test_token".to_string(),
            symbol: "TNT".to_string(),
            native_denom: "uaura".to_string(),
        };

        assert_eq!(msg.get_cap(), Some(Uint128::from(1u128)))
    }
}
