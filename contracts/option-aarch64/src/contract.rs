#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, BankMsg};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{UnderwriteOptionRequest, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{OptionStatus, State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:options";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if info.funds != msg.collateral {
        return Err(ContractError::Unauthorized {});
    }
    let state = State {
        option_status: OptionStatus::CREATED,
        creator: info.sender.clone(),
        holder : info.sender.clone(),
        underwriter: None,
        asset: msg.asset,
        premium: msg.premium,
        collateral: msg.collateral,
        expires: msg.expires,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("creator", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::FundPremium {} => fund_premium(deps, env, info),
        ExecuteMsg::TransferOption { recipient } => transfer_option(deps, env, info, recipient),
        ExecuteMsg::UnderwriteOption { underwrite_option_req } => underwrite_option(deps, env, info, underwrite_option_req),
        ExecuteMsg::ExecuteOption {} => execute_option(deps, env, info),
        ExecuteMsg::WithdrawExpiredOption {} => withdraw_expired_option(deps, env, info),
        ExecuteMsg::WithdrawUnlockedOption {} => withdraw_unlocked_option(deps, env, info),
    }
}
pub fn fund_premium(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if state.holder != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if state.premium != info.funds {
        return Err(ContractError::PremiumPriceMismatch {
            offer: info.funds,
            requires: state.premium
        });
    }

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.option_status = OptionStatus::FUNDED;
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("method", "fund_premium")
        .add_attribute("message", "successfully funded premium")
    )
}

pub fn underwrite_option(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    underwrite_option_req: UnderwriteOptionRequest) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if state.option_status != OptionStatus::FUNDED {
        return Err(ContractError::Unauthorized {});
    }

    // ensure the option-aarch64 has not expired
    if env.block.height < state.expires {
        return Err(ContractError::OptionExpired {
            expired: state.expires,
        });
    }

    if underwrite_option_req.premium != state.premium ||
        underwrite_option_req.collateral != state.collateral ||
        underwrite_option_req.asset != state.asset ||
        underwrite_option_req.expires != state.expires {
        return Err(ContractError::AgreementMismatch {})
    }

    if info.funds != state.asset {
        return Err(ContractError::AssetPriceMismatch {
            offer: info.funds,
            requires: state.asset
        })
    }

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.underwriter = Option::from(info.sender.clone());
        state.option_status = OptionStatus::LOCKED;
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("method", "write_option")
        .add_attribute("message", "successfully ")
    )
}

pub fn transfer_option(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Addr) -> Result<Response, ContractError> {

    let state = STATE.load(deps.storage)?;

    if info.sender != state.holder {
        return Err(ContractError::Unauthorized {});
    }
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.holder = recipient.clone();
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("method", "handle_transfer")
        .add_attribute("recipient", recipient.clone())
    )
}

pub fn execute_option(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    // ensure sender is the owner
    if info.sender != state.holder {
        return Err(ContractError::Unauthorized {});
    }

    // ensure the option-aarch64 has not expired
    if env.block.height < state.expires {
        return Err(ContractError::OptionExpired {
            expired: state.expires,
        });
    }

    if state.option_status != OptionStatus::LOCKED {
        return Err(ContractError::Unauthorized {});
    }

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.option_status = OptionStatus::EXECUTED;
        Ok(state)
    })?;

    let response = Response::new()
        .add_message(
        BankMsg::Send {
            to_address: state.holder.clone().into_string(),
            amount: state.asset,
    }).add_message(
        BankMsg::Send {
            to_address: state.underwriter.unwrap().clone().into_string(),
            amount: state.collateral,
    });

    Ok(response)
}

pub fn withdraw_expired_option(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if env.block.height < state.expires {
        return Err(ContractError::Unauthorized {})
    }

    Ok(Response::new()
        .add_message(
            BankMsg::Send {
                to_address: state.holder.into_string(),
                amount: state.collateral,
            })
        .add_message(
            BankMsg::Send {
            to_address: state.underwriter.unwrap().into_string(),
                amount: state.asset,
        })
    )
}

pub fn withdraw_unlocked_option(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if state.option_status != OptionStatus::CREATED ||
        state.holder != info.sender.clone()
        {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new())
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOptionContract {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(ConfigResponse{state})
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Coin, coins, from_binary, Uint128};

    #[test]
    fn pass_proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("creator", &coins(10, "uluna"));
        let msg = InstantiateMsg {
            asset: coins(10, "uusd"),
            collateral: coins(10, "uluna"),
            premium: coins(1, "uusd"),
            expires: 10000
        };

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOptionContract {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(info.sender, value.state.holder);
        assert_eq!(info.sender, value.state.creator);
        assert_eq!(None, value.state.underwriter);
        assert_eq!(msg.asset, value.state.asset);
        assert_eq!(msg.collateral, value.state.collateral);
        assert_eq!(msg.premium, value.state.premium);
        assert_eq!(msg.expires, value.state.expires);
    }

    #[test]
    fn pass_fund_premium_as_creator() {
        let mut deps = mock_dependencies(&[]);

        let info = mock_info("holder", &coins(10, "uluna"));
        let msg = InstantiateMsg {
            asset: coins(10, "uusd"),
            collateral: coins(10, "uluna"),
            premium: coins(1, "uusd"),
            expires: 10000
        };
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        // fund the premium for the option-aarch64 contract with the correct amount and denomination
        let info = mock_info("holder", &coins(1, "uusd"));
        let msg = ExecuteMsg::FundPremium {};
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Status should be FUNDED
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOptionContract {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(OptionStatus::FUNDED, value.state.option_status);
    }

    #[test]
    fn fail_fund_premium_as_creator() {

        let mut deps = mock_dependencies(&[]);
        let info = mock_info("holder", &coins(10, "uluna"));
        let msg = InstantiateMsg {
            asset: coins(10, "uusd"),
            collateral: coins(10, "uluna"),
            premium: coins(1, "uusd"),
            expires: 10000
        };
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        // fund the premium for the option-aarch64 contract with the wrong amount and denomination
        let info = mock_info("holder", &coins(10, "token"));
        let msg = ExecuteMsg::FundPremium {};
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        match res {
            Ok(_response) => assert!(false),
            Err(_e) => assert!(true)
        }
    }

    #[test]
    fn pass_underwrite_option() {
        let mut deps = mock_dependencies(&[]);

        let info = mock_info("holder", &coins(10, "uluna"));
        let msg = InstantiateMsg {
            asset: coins(10, "uusd"),
            collateral: coins(10, "uluna"),
            premium: coins(1, "uusd"),
            expires: 10000
        };
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();

        // fund the premium for the option-aarch64 contract with the correct amount and denomination
        let info = mock_info("holder", &coins(1, "uusd"));
        let msg = ExecuteMsg::FundPremium {};
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // underwrite the option-aarch64 contract
        let info = mock_info("underwriter", &coins(10, "uusd"));
        let underwrite_option_req = UnderwriteOptionRequest {
            asset: coins(10, "uusd"),
            collateral: coins(10, "uluna"),
            premium: coins(1, "uusd"),
            expires: 10000
        };
        let msg = ExecuteMsg::UnderwriteOption {
            underwrite_option_req,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOptionContract {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(OptionStatus::LOCKED, value.state.option_status);
        assert_eq!(info.sender, value.state.underwriter.unwrap());
    }

    #[test]
    fn pass_execute_option() {
        let mut deps = mock_dependencies(&[]);
        let env = mock_env();
        let holder_info = mock_info("holder", &coins(10, "uluna"));
        let msg = InstantiateMsg {
            asset: coins(10, "uusd"),
            collateral: coins(10, "uluna"),
            premium: coins(1, "uusd"),
            expires: 10000
        };
        instantiate(deps.as_mut(), env, holder_info, msg.clone()).unwrap();

        // fund the premium for the option-aarch64 contract with the correct amount and denomination
        let holder_info = mock_info("holder", &coins(1, "uusd"));
        let msg = ExecuteMsg::FundPremium {};
        execute(deps.as_mut(), env.clone(), holder_info, msg).unwrap();

        // underwrite the option-aarch64 contract
        let underwriter_info = mock_info("underwriter", &coins(10, "uusd"));
        let underwrite_option_req = UnderwriteOptionRequest {
            asset: coins(10, "uusd"),
            collateral: coins(10, "uluna"),
            premium: coins(1, "uusd"),
            expires: 10000
        };
        let msg = ExecuteMsg::UnderwriteOption {
            underwrite_option_req,
        };
        execute(deps.as_mut(), env.clone(), underwriter_info.clone(), msg);


        // Check contract balance
        let balance_amount = &deps.as_mut().querier.query_balance(env.clone().contract.address, "uusd".to_string()).unwrap();
        assert_eq!(Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(11 as u128)
        }, *balance_amount);
        let balance_amount = &deps.as_mut().querier.query_balance(env.clone().contract.address, "uluna".to_string()).unwrap();
        assert_eq!(Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(10 as u128)
        }, *balance_amount);

        // execute the option-aarch64 contract
        let holder_info = mock_info("holder", &[]);
        let msg = ExecuteMsg::ExecuteOption {};
        execute(deps.as_mut(), env.clone(), holder_info.clone(), msg);

        let res = query(deps.as_ref(), env.clone(), QueryMsg::GetOptionContract {}).unwrap();
        let value: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(OptionStatus::EXECUTED, value.state.option_status);

        // Check contract balance
        let balance_amount = &deps.as_mut().querier.query_balance(env.clone().contract.address, "uusd".to_string()).unwrap();
        assert_eq!(Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(0 as u128)
        }, *balance_amount);
        let balance_amount = &deps.as_mut().querier.query_balance(env.clone().contract.address, "uluna".to_string()).unwrap();
        assert_eq!(Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(0 as u128)
        }, *balance_amount);

        // Check holder balance
        let balance_amount = &deps.as_mut().querier.query_balance(holder_info.sender, "uusd".to_string()).unwrap();
        assert_eq!(Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(10 as u128)
        }, *balance_amount);

        // Check underwriter balance
        let balance_amount = &deps.as_mut().querier.query_balance(underwriter_info.clone().sender, "uluna".to_string()).unwrap();
        assert_eq!(Coin {
            denom: "uluna".to_string(),
            amount: Uint128::new(10 as u128)
        }, *balance_amount);

        let balance_amount = &deps.as_mut().querier.query_balance(underwriter_info.sender, "uusd".to_string()).unwrap();
        assert_eq!(Coin {
            denom: "uusd".to_string(),
            amount: Uint128::new(1 as u128)
        }, *balance_amount)
    }
}
