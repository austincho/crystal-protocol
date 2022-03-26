use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{ State};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub asset: Vec<Coin>,
    pub collateral: Vec<Coin>,
    pub premium: Vec<Coin>,
    pub expires: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    TransferOption {recipient : Addr},
    FundCollateral{},
    FundPremium {},
    UnderwriteOption { underwrite_option_req: UnderwriteOptionRequest },
    ExecuteOption {},
    WithdrawExpiredOption {},
    WithdrawUnlockedOption {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetOptionContract {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub(crate) state: State
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnderwriteOptionRequest {
    pub asset: Vec<Coin>,
    pub collateral: Vec<Coin>,
    pub premium: Vec<Coin>,
    pub expires: u64,
}

