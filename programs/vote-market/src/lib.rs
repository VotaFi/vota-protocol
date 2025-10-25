pub mod errors;
pub mod state;
pub mod util;
pub mod instructions;

use anchor_lang::prelude::*;

use instructions::*;

declare_id!("VotAjwzAEF9ZLNAYEB1ivXt51911EqYGVu9NeaEKRyy");

#[program]
pub mod vote_market {
    use super::*;

    pub fn create_config(
        ctx: Context<CreateConfig>,
        mints: Vec<Pubkey>,
        claim_fee: u16,
        script_authority: Pubkey,
    ) -> Result<()> {
        instructions::create_config::create_config(ctx, mints, claim_fee, script_authority)
    }

    pub fn update_reward_accumulator_config(
        ctx: Context<CreateRewardAccumulatorConfig>,
        reward_accumulator_program: Pubkey,
        namespace: [u8; 8]
    ) -> Result<()> {
        instructions::update_reward_accumulator_config::update_reward_accumulator_config(ctx, reward_accumulator_program, namespace)
    }

    pub fn update_admin(ctx: Context<UpdateAdmin>, admin: Pubkey) -> Result<()> {
        instructions::update_admin::update_admin(ctx, admin)
    }

    pub fn update_claim_fee(ctx: Context<UpdateClaimFee>, claim_fee: u16) -> Result<()> {
        instructions::update_claim_fee::update_claim_fee(ctx, claim_fee)
    }

    pub fn update_script_authority(
        ctx: Context<UpdateScriptAuthority>,
        script_authority: Pubkey,
    ) -> Result<()> {
        instructions::update_script_authority::update_script_authority(ctx, script_authority)
    }

    pub fn update_allowed_mints(
        ctx: Context<UpdateAllowedMints>,
        allowed_mints: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::update_allowed_mints::update_allowed_mints(ctx, allowed_mints)
    }

    pub fn increase_vote_buy(ctx: Context<IncreaseVoteBuy>, epoch: u32, amount: u64) -> Result<()> {
        instructions::increase_vote_buy::increase_vote_buy(ctx, epoch, amount)
    }

    pub fn claim_vote_payment(ctx: Context<ClaimVotePayment>, epoch: u32) -> Result<()> {
        instructions::claim_vote_payment::claim_vote_payment(ctx, epoch)
    }

    pub fn vote(ctx: Context<Vote>, weight: u32) -> Result<()> {
        instructions::vote::vote(ctx, weight)
    }

    pub fn claim_to_reward_accumulator(ctx: Context<ClaimToRewardAccumulator>, epoch: u32) -> Result<()> {
        instructions::claim_to_reward_accumulator::claim_to_reward_accumulator(ctx, epoch)
    }

    pub fn commit_vote(ctx: Context<CommitVote>, epoch: u32) -> Result<()> {
        instructions::commit_vote::commit_vote(ctx, epoch)
    }

    pub fn set_max_amount(ctx: Context<SetMaxAmount>, epoch: u32, max_amount: u64) -> Result<()> {
        instructions::set_max_amount::set_max_amount(ctx, epoch, max_amount)
    }

    pub fn vote_buy_refund(ctx: Context<VoteBuyRefund>, epoch: u32) -> Result<()> {
        instructions::vote_buy_refund::vote_buy_refund(ctx, epoch)
    }
}
