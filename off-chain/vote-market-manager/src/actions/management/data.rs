use crate::actions::management::oracle::KnownTokens;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EpochData {
    #[serde(
        deserialize_with = "common::deserialize_pubkey",
        serialize_with = "common::serialize_pubkey"
    )]
    pub config: Pubkey,
    pub epoch: u32,
    pub total_votes: u64,
    pub direct_votes: u64,
    pub delegated_votes: u64,
    pub total_vote_buy_value: f64,
    pub gauges: Vec<GaugeInfo>,
    pub prices: HashMap<KnownTokens, f64>,
    pub sbr_per_epoch: u64,
    #[serde(
        deserialize_with = "common::deserialize_pubkey_vec",
        serialize_with = "common::serialize_pubkey_vec"
    )]
    pub escrow_owners: Vec<Pubkey>,
    pub usd_per_vote: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GaugeInfo {
    #[serde(
        deserialize_with = "common::deserialize_pubkey",
        serialize_with = "common::serialize_pubkey"
    )]
    pub gauge: Pubkey,
    pub payment: f64,
    pub votes: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoteInfo {
    #[serde(
        deserialize_with = "common::deserialize_pubkey",
        serialize_with = "common::serialize_pubkey"
    )]
    pub gauge: Pubkey,
    pub votes: u64,
    pub weight: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoteInfoCollection(pub Vec<VoteInfo>);
