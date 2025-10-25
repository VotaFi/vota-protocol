use anchor_lang::prelude::*;
use crate::state::{VoteMarketConfig, RewardAccumulatorConfig};

#[derive(Accounts)]
pub struct CreateRewardAccumulatorConfig<'info> {
    #[account(has_one = admin)]
    pub config: Account<'info, VoteMarketConfig>,
    #[account(init_if_needed,
    payer = admin,
    space = VoteMarketConfig::LEN,
    seeds = [b"reward-accumulator-config".as_ref(), config.to_account_info().key.as_ref()],
        bump)]
    pub reward_accumulator_config: Account<'info, RewardAccumulatorConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn update_reward_accumulator_config(
    ctx: Context<CreateRewardAccumulatorConfig>,
    reward_accumulator_program: Pubkey,
    namespace: [u8; 8]
) -> Result<()> {
    let cfg = &mut ctx.accounts.reward_accumulator_config;
    cfg.reward_accumulator_program = reward_accumulator_program;
    cfg.namespace = namespace;
    Ok(())
}

