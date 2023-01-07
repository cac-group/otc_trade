use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("No offer made")]
    NoFunds,

    #[error("OTC deal is already closed")]
    ContractClosed,

    #[error("Offer failed, must be equal or higher than asking price")]
    OfferFail,

    #[error("You can't close the contract. Only {owner} can")]
    NotOwner { owner: String },
}
