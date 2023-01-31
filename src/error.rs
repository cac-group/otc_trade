use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Not enough funds")]
    NoFunds,

    #[error("You can only offer 1 asset")]
    NotOneAsset,

    #[error("No cw20 contract and value sent")]
    NoContract,

    #[error("Contract already closed")]
    ContractClosed,

    #[error("You can't close the contract. Only {owner} can")]
    NotOwner { owner: String },
}
