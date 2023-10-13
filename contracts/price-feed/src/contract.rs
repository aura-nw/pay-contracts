#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{RoundData, PRICE_FEED_INFO, ROUND_DATA};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:price-feed";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateRoundData { answer } => update_round_data(deps, env, info, answer),
    }
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastestRoundData {} => to_binary(&query_lastest_round_data(deps)?),
    }
}

pub fn update_round_data(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    answer: u64,
) -> Result<Response, ContractError> {
    // check if the sender is the owner
    let config = crate::state::CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // load the latest round id from the price feed info
    let latest_round_id = PRICE_FEED_INFO.load(deps.storage)?.latest_round;
    // load the round data from the round data map
    let mut lastest_round_data = ROUND_DATA.load(deps.storage, latest_round_id)?;
    // update the round data
    // TODO: update the other fields
    lastest_round_data.answer = Uint128::from(answer);
    // save the round data
    ROUND_DATA.save(deps.storage, latest_round_id, &lastest_round_data)?;

    // return the response
    Ok(Response::new()
        .add_attribute("method", "update_round_data")
        .add_attribute("round_id", latest_round_id.to_string())
        .add_attribute("answer", answer.to_string()))
}

pub fn query_lastest_round_data(deps: Deps) -> StdResult<RoundData> {
    // load the latest round id from the price feed info
    let latest_round_id = PRICE_FEED_INFO.load(deps.storage)?.latest_round;
    // load the round data from the round data map
    let lastest_round_data = ROUND_DATA.load(deps.storage, latest_round_id)?;
    // return the round data
    Ok(lastest_round_data)
}
