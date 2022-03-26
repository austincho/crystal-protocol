use cosmwasm_std::{Coin, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Expired")]
    OptionExpired {
        expired: u64
    },

    #[error("Premium Price Mismatch")]
    PremiumPriceMismatch {
        offer: Vec<Coin>,
        requires: Vec<Coin>
    },

    #[error("Asset Price Mismatch")]
    AssetPriceMismatch {
        offer: Vec<Coin>,
        requires: Vec<Coin>
    },

    #[error("Option Terms Does Not Match Created Option")]
    AgreementMismatch {},


    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
