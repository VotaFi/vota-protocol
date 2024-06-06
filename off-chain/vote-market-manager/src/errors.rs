use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum VoteMarketManagerError {
    AddressNotFound,
    PriorityFeeNotInResult,
    #[allow(dead_code)]
    BlockHashExpired,
    #[allow(dead_code)]
    SimulationFailed,
}

impl std::error::Error for VoteMarketManagerError {}
impl Display for VoteMarketManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoteMarketManagerError::AddressNotFound => write!(f, "Address not found"),
            VoteMarketManagerError::PriorityFeeNotInResult => {
                write!(f, "Priority fee not in result")
            }
            VoteMarketManagerError::BlockHashExpired => write!(f, "Block hash expired"),
            VoteMarketManagerError::SimulationFailed => write!(f, "Simulation failed"),
        }
    }
}
