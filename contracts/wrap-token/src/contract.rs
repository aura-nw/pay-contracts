#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};

use cw2::set_contract_version;

use cw20::Cw20Coin;
use cw20_base::contract::{execute as cw20_execute, query as cw20_query};
use cw20_base::msg::{ExecuteMsg, QueryMsg};
use cw20_base::state::{MinterData, TokenInfo, BALANCES, TOKEN_INFO};
use cw20_base::ContractError;

use crate::state::{InstantiateMsg, SupportedNative, SUPPORTED_NATIVE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wrap-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// this is the denominator of native token that is supported by this contract
pub static NATIVE_DENOM: &str = "uaura";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // check valid token info
    msg.validate()?;

    // this is a sanity check, to ensure that each token of this contract has garanteed by 1 native token
    if !msg.initial_balances.is_empty() {
        return Err(StdError::generic_err("Initial balances must be empty").into());
    }

    let init_supply = Uint128::zero();

    if let Some(limit) = msg.get_cap() {
        if init_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: init_supply,
        mint,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    SUPPORTED_NATIVE.save(
        deps.storage,
        &SupportedNative {
            denom: msg.native_denom,
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Mint {
            recipient,
            amount: _,
        } => execute_mint(deps, env, info, recipient),
        _ => cw20_execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_query(deps, env, msg)
}

fn validate_balance(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let native_denom = SUPPORTED_NATIVE.load(deps.storage)?.denom;
    let total_supply = TOKEN_INFO.load(deps.storage)?.total_supply;
    let balance = deps
        .querier
        .query_balance(env.contract.address, native_denom)
        .unwrap()
        .amount;

    if balance < total_supply {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Invalid balance, {}, {}",
            total_supply, balance,
        ))));
    }
    Ok(Response::new())
}

// After a user burn the token, contract will return the same amount of native token to him
// This function is taken from cw20-base with some modifications
pub fn execute_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // lower balance of sender
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    let native_denom = SUPPORTED_NATIVE.load(deps.storage)?.denom;
    // transfer native tokens to sender
    let transfer_native_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: native_denom,
            amount,
        }],
    };

    Ok(Response::new()
        .add_message(transfer_native_msg)
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount))
}

// Every user send native token to this contract, and the contract will mint the same amount of token to the user.
// This function is taken from cw20-base with some modifications
pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
) -> Result<Response, ContractError> {
    let mut config = TOKEN_INFO
        .may_load(deps.storage)?
        .ok_or(ContractError::Unauthorized {})?;

    if config
        .mint
        .as_ref()
        .ok_or(ContractError::Unauthorized {})?
        .minter
        != info.sender
    {
        return Err(ContractError::Unauthorized {});
    }

    // check the funds are sent with the message
    // if the denom of funds is not the same as the native denom, we reject
    let native_denom = SUPPORTED_NATIVE.load(deps.storage)?.denom;
    if info.funds.len() != 1
        || info.funds[0].denom != native_denom
        || info.funds[0].amount == Uint128::zero()
    {
        return Err(ContractError::Unauthorized {});
    }

    // update supply and enforce cap
    config.total_supply += info.funds[0].amount;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &config)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + info.funds[0].amount)
        },
    )?;

    validate_balance(deps, env).unwrap();

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", recipient)
        .add_attribute("amount", info.funds[0].amount))
}

pub fn create_accounts(
    deps: &mut DepsMut,
    accounts: &[Cw20Coin],
) -> Result<Uint128, ContractError> {
    validate_accounts(accounts)?;

    let mut total_supply = Uint128::zero();
    for row in accounts {
        let address = deps.api.addr_validate(&row.address)?;
        BALANCES.save(deps.storage, &address, &row.amount)?;
        total_supply += row.amount;
    }

    Ok(total_supply)
}

pub fn validate_accounts(accounts: &[Cw20Coin]) -> Result<(), ContractError> {
    let mut addresses = accounts.iter().map(|c| &c.address).collect::<Vec<_>>();
    addresses.sort();
    addresses.dedup();

    if addresses.len() != accounts.len() {
        Err(ContractError::DuplicateInitialBalanceAddresses {})
    } else {
        Ok(())
    }
}
