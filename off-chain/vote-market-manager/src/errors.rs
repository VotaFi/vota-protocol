use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum VoteMarketManagerError {
    AddressNotFound,
    SimulationFailed {
        sim_info: String,
    },
    SendTransactionError {
       message: String,
    }
}

impl std::error::Error for VoteMarketManagerError {}
impl Display for VoteMarketManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoteMarketManagerError::AddressNotFound => write!(f, "Address not found"),
            VoteMarketManagerError::SimulationFailed { sim_info } => {
                write!(f, "Simulation failed: {}", sim_info)
            },
            VoteMarketManagerError::SendTransactionError { message } => {
                write!(f, "Error sending transaction: {}", message)
            }
        }
    }
}
