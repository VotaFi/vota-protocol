use anchor_lang::prelude::*;
use crate::state::{VoteMarketConfig, VoteBuy};

#[derive(Accounts)]
#[instruction(epoch: u32)]
pub struct SetMaxAmount<'info> {
    pub config: Account<'info, VoteMarketConfig>,
    // Need to verify seeds to make sure the correct script_authority is used
    #[account(mut,
    seeds = [
        b"vote-buy".as_ref(),
        epoch.to_le_bytes().as_ref(),
        config.key().as_ref(),
        gauge.key().as_ref()], bump)]
    pub vote_buy: Account<'info, VoteBuy>,
    #[account(
    constraint = config.gaugemeister == gauge.gaugemeister,
    constraint = !gauge.is_disabled)]
    pub gauge: Account<'info, gauge_state::Gauge>,
    #[account(address = config.script_authority)]
    pub script_authority: Signer<'info>,
}

#[allow(unused_variables)]
pub fn set_max_amount(ctx: Context<SetMaxAmount>, epoch: u32, max_amount: u64) -> Result<()> {
    ctx.accounts.vote_buy.max_amount = Some(max_amount);
    Ok(())
}

