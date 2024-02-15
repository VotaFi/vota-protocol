use anchor_lang::prelude::*;

declare_id!("4MMZH3ih1aSty2nx4MC3kSR94Zb55XsXnqb5jfEcyHWQ");
pub const SECONDS_PER_YEAR: u128 = 86_400 * 365;

/// Controls token rewards distribution to all [Quarry]s.
/// The [Rewarder] is also the [quarry_mint_wrapper::Minter] registered to the [quarry_mint_wrapper::MintWrapper].
#[account]
#[derive(Copy, Default, Debug)]
pub struct Rewarder {
    /// Random pubkey used for generating the program address.
    pub base: Pubkey,
    /// Bump seed for program address.
    pub bump: u8,

    /// Authority who controls the rewarder
    pub authority: Pubkey,
    /// Pending authority which must accept the authority
    pub pending_authority: Pubkey,

    /// Number of [Quarry]s the [Rewarder] manages.
    /// If more than this many [Quarry]s are desired, one can create
    /// a second rewarder.
    pub num_quarries: u16,
    /// Amount of reward tokens distributed per day
    pub annual_rewards_rate: u64,
    /// Total amount of rewards shares allocated to [Quarry]s
    pub total_rewards_shares: u64,
    /// Mint wrapper.
    pub mint_wrapper: Pubkey,
    /// Mint of the rewards token for this [Rewarder].
    pub rewards_token_mint: Pubkey,

    /// Claim fees are placed in this account.
    pub claim_fee_token_account: Pubkey,
    /// Maximum amount of tokens to send to the Quarry DAO on each claim,
    /// in terms of milliBPS. 1,000 milliBPS = 1 BPS = 0.01%
    /// This is stored on the [Rewarder] to ensure that the fee will
    /// not exceed this in the future.
    pub max_claim_fee_millibps: u64,

    /// Authority allowed to pause a [Rewarder].
    pub pause_authority: Pubkey,
    /// If true, all instructions on the [Rewarder] are paused other than [quarry_mine::unpause].
    pub is_paused: bool,
}
impl Rewarder {
    pub const LEN: usize = 32 + 1 + 32 + 32 + 2 + 8 + 8 + 32 + 32 + 32 + 8 + 32 + 1;

}
