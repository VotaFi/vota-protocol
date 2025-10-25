use anchor_lang::prelude::*;
use crate::state::VoteMarketConfig;

#[derive(Accounts)]
pub struct UpdateClaimFee<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, VoteMarketConfig>,
    pub admin: Signer<'info>,
}

pub fn update_claim_fee(ctx: Context<UpdateClaimFee>, claim_fee: u16) -> Result<()> {
    ctx.accounts.config.claim_fee = claim_fee;
    Ok(())
}

