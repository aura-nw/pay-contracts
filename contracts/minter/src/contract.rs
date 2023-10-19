#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, Addr, BalanceResponse, BankMsg, BankQuery, Binary, Coin, CosmosMsg, Deps,
    DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::MinterResponse;
use cw20_base::msg::{ExecuteMsg as Cw20ExecuteMsg, InstantiateMsg as Cw20InstantiateMsg};
use cw_utils::parse_reply_instantiate_data;
use price_feed::msg::{QueryMsg as PriceFeedQueryMsg, RoundDataResponse};

use crate::error::ContractError;
use crate::msg::{ExchangingInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ReceiverResponse};
use crate::state::{Config, ExchangingInfo, EXCHANGING_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    let config = Config {
        owner: info.sender.clone(),
        receiver_name: msg.receiver_name.clone(),
    };
    crate::state::CONFIG.save(deps.storage, &config)?;

    // init receiver info
    let receiver_info = ExchangingInfo {
        accepted_denom: msg.accepted_denom.clone(),
        receiver_address: deps.api.addr_validate(&msg.receiver_address)?,
        token_address: Addr::unchecked("default".to_string()),
        price_feed: deps.api.addr_validate(&msg.price_feed)?,
    };
    EXCHANGING_INFO.save(deps.storage, &receiver_info)?;

    let new_token_instantiation_msg = Cw20InstantiateMsg {
        mint: Some(MinterResponse {
            minter: env.contract.address.to_string(),
            cap: None,
        }),
        ..msg.token_instantiation_msg
    };

    // now we instantiate the cw20 contract
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.clone())
        .add_attribute("token_code_id", msg.token_code_id.to_string())
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(info.sender.to_string()),
                code_id: msg.token_code_id,
                msg: to_binary(&new_token_instantiation_msg)?,
                funds: vec![],
                label: "Intantiate token for minter".to_string(),
            }),
            reply_on: ReplyOn::Success,
        }))
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
        ExecuteMsg::Exchange {
            amount,
            expected_received,
        } => execute_exchange(deps, env, info, amount, expected_received),
        ExecuteMsg::Withdraw {} => execute_withdraw(deps, env, info),
    }
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Owner {} => to_binary(&query_owner(deps)?),
        QueryMsg::Receiver {} => to_binary(&query_receiver(deps)?),
        QueryMsg::ExchangingInfo {} => to_binary(&query_exchanging_info(deps)?),
    }
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    // if code_id is not 1, it's not a reply from cw20 contract
    if msg.id != 1 {
        return Err(ContractError::Unauthorized {});
    }
    let reply_msg = parse_reply_instantiate_data(msg).unwrap();

    // load receiver info
    let mut receiver_info = EXCHANGING_INFO.load(deps.storage)?;
    receiver_info.token_address = deps
        .api
        .addr_validate(&reply_msg.contract_address.clone())?;

    // save receiver info
    EXCHANGING_INFO.save(deps.storage, &receiver_info)?;

    Ok(Response::new()
        .add_attribute("method", "reply")
        .add_attribute("token_address", reply_msg.contract_address))
}

pub fn execute_exchange(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    expected_received: Uint128,
) -> Result<Response, ContractError> {
    // the funds must have enough offer token
    let exchanging_info = EXCHANGING_INFO.load(deps.storage)?;
    let offer_token = Coin {
        denom: exchanging_info.accepted_denom,
        amount,
    };
    if !has_coins(&info.funds, &offer_token) {
        return Err(ContractError::NotEnoughFunds {});
    }

    // query last round data from price feed
    // query the owner of the nft
    let lastest_round_data: RoundDataResponse = deps
        .querier
        .query_wasm_smart(
            exchanging_info.price_feed,
            &PriceFeedQueryMsg::LastestRoundData {},
        )
        .unwrap();

    // calculate the amount of stable token to be minted
    let stable_token_amount = amount * lastest_round_data.answer;
    if stable_token_amount < expected_received {
        return Err(ContractError::ExpectedReceivedNotMatched {});
    }

    // mint stable token to receiver
    let mint_msg = WasmMsg::Execute {
        contract_addr: exchanging_info.token_address.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: exchanging_info.receiver_address.to_string(),
            amount: expected_received,
        })?,
        funds: vec![],
    };

    // send the exchange message to the cw20 contract
    Ok(Response::new().add_message(mint_msg).add_attributes([
        ("method", "exchange"),
        ("amount", &amount.to_string()),
        ("expected_received", &expected_received.to_string()),
    ]))
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // only owner can withdraw
    let config = crate::state::CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let exchange_info = EXCHANGING_INFO.load(deps.storage)?;
    // query the balance of the contract
    let balance: BalanceResponse = deps
        .querier
        .query(&cosmwasm_std::QueryRequest::Bank(BankQuery::Balance {
            address: env.contract.address.to_string(),
            denom: exchange_info.accepted_denom.clone(),
        }))
        .unwrap();

    // transfer the balance to the owner
    let transfer_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: exchange_info.accepted_denom,
            amount: balance.amount.amount,
        }],
    };

    Ok(Response::new().add_message(transfer_msg).add_attributes([
        ("method", "withdraw"),
        ("amount", &balance.amount.amount.to_string()),
    ]))
}

pub fn query_owner(deps: Deps) -> StdResult<String> {
    let config = crate::state::CONFIG.load(deps.storage)?;
    Ok(config.owner.to_string())
}

pub fn query_receiver(deps: Deps) -> StdResult<ReceiverResponse> {
    let config = crate::state::CONFIG.load(deps.storage)?;
    let exchange_info = EXCHANGING_INFO.load(deps.storage)?;
    Ok(ReceiverResponse {
        name: config.receiver_name,
        address: exchange_info.receiver_address.to_string(),
    })
}

pub fn query_exchanging_info(deps: Deps) -> StdResult<ExchangingInfoResponse> {
    let exchanging_info = EXCHANGING_INFO.load(deps.storage)?;
    Ok(ExchangingInfoResponse {
        accepted_denom: exchanging_info.accepted_denom,
        token_address: exchanging_info.token_address.to_string(),
        price_feed: exchanging_info.price_feed.to_string(),
    })
}
