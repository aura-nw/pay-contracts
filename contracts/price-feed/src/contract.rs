#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RoundDataResponse};
use crate::state::{RoundData, PRICE_FEED_INFO, ROUND_DATA};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:price-feed";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// const MAX_ROUND_DATA_LEN: usize = 105120; // we will update answer every 5 minutes, so 105120 = 365 * 24 * 12
const MAX_DIFF_ROUND_ID: u64 = 60; // the max height diff between 2 round ids, 60 = 12 * 5

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
        ExecuteMsg::UpdateController { controller } => {
            update_controller(deps, env, info, controller)
        }
        ExecuteMsg::UpdateRoundData { answer } => update_round_data(deps, env, info, answer),
    }
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastestRoundData {} => to_binary(&query_lastest_round_data(deps, env)?),
        QueryMsg::RoundData { round_id } => to_binary(&query_round_data(deps, round_id)?),
    }
}

pub fn update_controller(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    controller: String,
) -> Result<Response, ContractError> {
    // check if the sender is the owner
    let config = crate::state::CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // update the controller
    let mut config = crate::state::CONFIG.load(deps.storage)?;
    config.controller = deps.api.addr_validate(&controller)?;
    crate::state::CONFIG.save(deps.storage, &config)?;

    // return the response
    Ok(Response::new()
        .add_attribute("method", "update_controller")
        .add_attribute("controller", controller))
}

pub fn update_round_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    answer: u64,
) -> Result<Response, ContractError> {
    // check if the sender is the controller
    let config = crate::state::CONFIG.load(deps.storage)?;
    if info.sender != config.controller {
        return Err(ContractError::Unauthorized {});
    }

    // // TODO: Optimize the code to reduce the gas cost
    // // remove the first round data if the round data length is greater than 105120
    // let round_data_len = ROUND_DATA
    //     .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
    //     .collect::<StdResult<Vec<_>>>()?;
    // if round_data_len.len() > MAX_ROUND_DATA_LEN {
    //     ROUND_DATA.remove(deps.storage, round_data_len[0].0);
    // }

    // load the latest round id from the price feed info
    let latest_round_id = env.block.height;

    // lastest round data
    let round_data = RoundData {
        answer: Uint128::from(answer),
        updated_at: env.block.time,
    };

    // save the round data
    ROUND_DATA.save(deps.storage, latest_round_id, &round_data)?;

    // update the latest round id in the price feed info
    let mut price_feed_info = PRICE_FEED_INFO.load(deps.storage)?;
    price_feed_info.latest_round = latest_round_id;
    PRICE_FEED_INFO.save(deps.storage, &price_feed_info)?;

    // return the response
    Ok(Response::new()
        .add_attribute("method", "update_round_data")
        .add_attribute("round_id", latest_round_id.to_string())
        .add_attribute("answer", answer.to_string()))
}

pub fn query_lastest_round_data(deps: Deps, env: Env) -> StdResult<RoundDataResponse> {
    // load the latest round id from the price feed info
    let latest_round_id = PRICE_FEED_INFO.load(deps.storage)?.latest_round;
    // load the round data from the round data map
    let lastest_round_data = ROUND_DATA.load(deps.storage, latest_round_id)?;

    let res = RoundDataResponse {
        round_id: env.block.height,
        answer: lastest_round_data.answer,
        updated_at: lastest_round_data.updated_at,
        answered_in_round: latest_round_id,
    };
    // return the round data
    Ok(res)
}

pub fn query_round_data(deps: Deps, round_id: u64) -> StdResult<RoundDataResponse> {
    let min: Option<Bound<u64>> = Some(Bound::inclusive(round_id - MAX_DIFF_ROUND_ID * 13)); // query data in previous 1 hour 5 minutes
    let max: Option<Bound<u64>> = Some(Bound::inclusive(round_id));
    // load the round data from the round data map
    let binding = ROUND_DATA
        .range(deps.storage, min, max, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    let round_data = binding.last().unwrap();

    let res = RoundDataResponse {
        round_id,
        answer: round_data.1.answer,
        updated_at: round_data.1.updated_at,
        answered_in_round: round_data.0,
    };
    // return the round data
    Ok(res)
}
