use anchor_lang::prelude::*;
use crate::state::VoteMarketConfig;

#[derive(Accounts)]
#[instruction(mints: Vec<Pubkey>)]
pub struct CreateConfig<'info> {
    #[account(
        init,
        payer = payer,
        space = VoteMarketConfig::LEN,
        )]
    pub config: Account<'info, VoteMarketConfig>,
    pub gaugemeister: Account<'info, gauge_state::Gaugemeister>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = crate::state::AllowedMints::len(mints.len()),
        seeds = [b"allow-list".as_ref(), config.to_account_info().key.as_ref()],
        bump)]
    pub allowed_mints: Account<'info, crate::state::AllowedMints>,
    pub system_program: Program<'info, System>,
}

pub fn create_config(
    ctx: Context<CreateConfig>,
    mints: Vec<Pubkey>,
    claim_fee: u16,
    script_authority: Pubkey,
) -> Result<()> {
    ctx.accounts.config.script_authority = script_authority;
    ctx.accounts.config.gaugemeister = ctx.accounts.gaugemeister.key();
    ctx.accounts.allowed_mints.mints = mints;
    ctx.accounts.config.admin = ctx.accounts.payer.key();
    ctx.accounts.config.claim_fee = claim_fee;
    Ok(())
}

