use anchor_lang::prelude::*;
use crate::state::VoteMarketConfig;

#[derive(Accounts)]
pub struct UpdateAdmin<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, VoteMarketConfig>,
    pub admin: Signer<'info>,
}

pub fn update_admin(ctx: Context<UpdateAdmin>, admin: Pubkey) -> Result<()> {
    ctx.accounts.config.admin = admin;
    Ok(())
}

