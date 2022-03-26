use schemars::{JsonSchema};
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub option_status: OptionStatus,
    pub creator: Addr,
    pub holder: Addr,
    pub underwriter: Option<Addr>,
    pub asset: Vec<Coin>,
    pub collateral: Vec<Coin>,
    pub premium: Vec<Coin>,
    pub expires: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum OptionStatus {
    CREATED,
    FUNDED,
    LOCKED,
    EXECUTED,
}

pub const STATE: Item<State> = Item::new("state");
