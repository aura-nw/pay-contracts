#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Answer, RoundData, CONFIG, FEEDERS, PRICE_FEED_INFO, ROUND_DATA};

use price_feed::msg::ExecuteMsg as PriceFeedExecuteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:price-collector";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DIFF_DURRATION: u64 = 300; // the max height diff between 2 round ids is 5 minutes, 300 = (12 * 5) * 5

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // init config
    let config = crate::state::Config {
        owner: info.sender.clone(),
    };
    CONFIG.save(deps.storage, &config)?;

    // init price feed info
    let price_feed_info = crate::state::PriceFeedInfo {
        price_feed: deps.api.addr_validate(&msg.price_feed)?,
        latest_round: env.block.height,
    };
    PRICE_FEED_INFO.save(deps.storage, &price_feed_info)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("controller", msg.price_feed))
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
        ExecuteMsg::UpdatePriceFeeder {
            price_feeder,
            status,
        } => update_price_feed(deps, env, info, price_feeder, status),
        ExecuteMsg::ProvideRoundData { answer } => update_round_data(deps, env, info, answer),
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

pub fn update_price_feed(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    price_feeder: String,
    status: bool,
) -> Result<Response, ContractError> {
    // check if the sender is the owner
    let config = crate::state::CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // update the status of the price feed in the list
    if status {
        FEEDERS.save(deps.storage, deps.api.addr_validate(&price_feeder)?, &true)?;
    } else {
        FEEDERS.remove(deps.storage, deps.api.addr_validate(&price_feeder)?);
    }

    // return the response
    Ok(Response::new().add_attributes([
        ("method", "update_price_feed"),
        ("price_feed", price_feeder.as_str()),
        ("status", status.to_string().as_str()),
    ]))
}

pub fn update_round_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    answer: u64,
) -> Result<Response, ContractError> {
    // only a feeder can call this method
    if !FEEDERS.has(deps.storage, info.sender.clone()) {
        return Err(ContractError::Unauthorized {});
    }

    // load latest round id
    let latest_round_id = PRICE_FEED_INFO.load(deps.storage)?.latest_round;

    if latest_round_id < env.block.height - MAX_DIFF_DURRATION {
        // The lastest round is not expired yet, so just add new answer to the round data
        // load the RoundData
        let mut round_data = ROUND_DATA.load(deps.storage, latest_round_id)?;

        // if the round data is ended
        if round_data.status != crate::state::RoundDataStatus::Pending {
            return Err(ContractError::RoundEnded {});
        }

        // if the round data is already answered by the sender
        if round_data.is_answered(info.sender.clone()) {
            return Err(ContractError::InvalidUpdate {});
        }

        // create new answer
        let new_answer = Answer {
            provider: info.sender,
            value: answer,
            updated_at_height: env.block.height,
        };

        round_data.answers.push(new_answer.clone());

        let mut res = Response::new();

        // if the number of answers is greater than 2/3 of the feeders,
        // then push the answer for price feed contract and the status of round is answered
        let feeders = FEEDERS
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        if round_data.current_number_answeres() * 3 > feeders.len() * 2 {
            let mut sorted_answers = vec![];
            // sort the answers by the answer value using insertion sort
            for answer in round_data.answers.iter() {
                let mut i = sorted_answers.len();
                while i > 0 && answer.value < sorted_answers[i - 1] {
                    i -= 1;
                }
                sorted_answers.insert(i, answer.value);
            }

            // the new answer must be not greater than 10% of the last answer and not less than 10% of the first answer
            if new_answer.value > sorted_answers[sorted_answers.len() - 1] * 11 / 10
                || new_answer.value < sorted_answers[0] * 9 / 10
            {
                // if the answer is not valid, then the round is rejected
                round_data.status = crate::state::RoundDataStatus::Rejected;
                round_data.answered_at_height = env.block.height;
            } else {
                // if the answer is valid, then the round is answered
                round_data.status = crate::state::RoundDataStatus::Answered;
                round_data.answered_at_height = env.block.height;

                // update the latest round id in the price feed info
                let price_feed_info = PRICE_FEED_INFO.load(deps.storage)?;
                // return the response
                res = res.add_message(WasmMsg::Execute {
                    contract_addr: price_feed_info.price_feed.to_string(),
                    msg: to_binary(&PriceFeedExecuteMsg::UpdateRoundData {
                        answer: round_data.current_answer(),
                    })?,
                    funds: vec![],
                })
            }
        }
        // just save the round data
        ROUND_DATA.save(deps.storage, latest_round_id, &round_data)?;

        Ok(res
            .add_attribute("method", "update_round_data")
            .add_attribute("round_id", latest_round_id.to_string())
            .add_attribute("answer", answer.to_string()))
    } else {
        // The lastest round is expired, so create new round data
        // create new round data
        let round_data = RoundData {
            started_at_height: env.block.height,
            status: crate::state::RoundDataStatus::Pending,
            answers: vec![Answer {
                provider: info.sender,
                value: answer,
                updated_at_height: env.block.height,
            }],
            answered_at_height: 0,
        };

        // save the round data
        ROUND_DATA.save(deps.storage, env.block.height, &round_data)?;

        // update the latest round id in the price feed info
        let mut price_feed_info = PRICE_FEED_INFO.load(deps.storage)?;
        price_feed_info.latest_round = env.block.height;
        PRICE_FEED_INFO.save(deps.storage, &price_feed_info)?;

        Ok(Response::new().add_attributes([
            ("method", "update_round_data"),
            ("round_id", env.block.height.to_string().as_str()),
            ("answer", answer.to_string().as_str()),
        ]))
    }
}

pub fn query_lastest_round_data(deps: Deps, _env: Env) -> StdResult<RoundData> {
    // load the latest round id from the price feed info
    let latest_round_id = PRICE_FEED_INFO.load(deps.storage)?.latest_round;
    // load the round data from the round data map
    let lastest_round_data = ROUND_DATA.load(deps.storage, latest_round_id)?;

    // return the round data
    Ok(lastest_round_data)
}

pub fn query_round_data(deps: Deps, round_id: u64) -> StdResult<RoundData> {
    let min: Option<Bound<u64>> = Some(Bound::inclusive(round_id - MAX_DIFF_DURRATION * 13)); // query data in previous 1 hour 5 minutes
    let max: Option<Bound<u64>> = Some(Bound::inclusive(round_id));
    // load the round data from the round data map
    let binding = ROUND_DATA
        .range(deps.storage, min, max, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    let round_data = binding.last().unwrap();

    let res = RoundData {
        started_at_height: round_data.1.started_at_height,
        status: round_data.1.status.clone(),
        answers: round_data.1.answers.clone(),
        answered_at_height: round_data.1.answered_at_height,
    };
    // return the round data
    Ok(res)
}
