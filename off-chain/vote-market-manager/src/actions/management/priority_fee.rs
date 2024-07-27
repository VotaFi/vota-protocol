use crate::errors::VoteMarketManagerError;
use helius::Helius;
use helius::types::{GetPriorityFeeEstimateOptions, GetPriorityFeeEstimateRequest, PriorityLevel};

pub async fn get_priority_fee(helius: &Helius) -> Result<f64, Box<dyn std::error::Error>> {
    let response = helius.rpc_client.get_priority_fee_estimate(GetPriorityFeeEstimateRequest {
        transaction: None,
        account_keys: Some(vec![gauge_state::id().to_string(),"JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string()]),
        options: Some(GetPriorityFeeEstimateOptions {
            priority_level: Some(PriorityLevel::Medium),
            include_all_priority_fee_levels: None,
            transaction_encoding: None,
            lookback_slots: None,
            recommended: None,
            include_vote: None,
        })
    }).await?;
    match response.priority_fee_estimate {
        Some(priority_fee_estimate) => Ok(priority_fee_estimate),
        None => Err(VoteMarketManagerError::PriorityFeeNotInResult.into()),
    }
}
