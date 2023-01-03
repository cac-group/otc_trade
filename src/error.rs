use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Bidding failed, must be higher than highest bid")]
    Biddingfail,

    #[error("Bidding failed, must make a bid")]
    BiddingEmpty,

    #[error("No bids have been made")]
    NoBids,

    #[error("Contract already closed")]
    ContractClosed,

    #[error("Contract not closed yet")]
    ContractNotClosed,

    #[error("Unauthorized: Only {owner} can call it.")]
    NotOwner { owner: String },
}