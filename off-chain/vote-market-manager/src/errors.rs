use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum VoteMarketManagerError {
    AddressNotFound,
    PriorityFeeNotInResult,
    SimulationFailed {
        sim_info: String,
    },
    DatabaseConnection {
        error: String,
    },
}

impl std::error::Error for VoteMarketManagerError {}
impl Display for VoteMarketManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoteMarketManagerError::AddressNotFound => write!(f, "Address not found"),
            VoteMarketManagerError::PriorityFeeNotInResult => {
                write!(f, "Priority fee not in result")
            },
            VoteMarketManagerError::SimulationFailed { sim_info } => {
                write!(f, "Simulation failed: {}", sim_info)
            },
            VoteMarketManagerError::DatabaseConnection { error } => {
                write!(f, "Database connection error: {}", error)
            },
        }
    }
}
