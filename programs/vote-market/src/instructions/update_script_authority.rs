use anchor_lang::prelude::*;
use crate::state::VoteMarketConfig;

#[derive(Accounts)]
pub struct UpdateScriptAuthority<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, VoteMarketConfig>,
    pub admin: Signer<'info>,
}

pub fn update_script_authority(
    ctx: Context<UpdateScriptAuthority>,
    script_authority: Pubkey,
) -> Result<()> {
    ctx.accounts.config.script_authority = script_authority;
    Ok(())
}

