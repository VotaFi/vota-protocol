use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use crate::state::{VoteMarketConfig, AllowedMints};

#[derive(Accounts)]
pub struct UpdateAllowedMints<'info> {
    #[account(mut, has_one = admin)]
    pub config: Account<'info, VoteMarketConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"allow-list".as_ref(), config.to_account_info().key.as_ref()],
        bump)]
    pub allowed_mints: Account<'info, AllowedMints>,
    pub system_program: Program<'info, System>,
}

pub fn update_allowed_mints(
    ctx: Context<UpdateAllowedMints>,
    allowed_mints: Vec<Pubkey>,
) -> Result<()> {
    let allowed_mints_size = AllowedMints::len(ctx.accounts.allowed_mints.mints.len());
    let next_allowed_mints_size = AllowedMints::len(allowed_mints.len());
    if next_allowed_mints_size > allowed_mints_size {
        let allowed_mints_account_info = ctx.accounts.allowed_mints.to_account_info();
        allowed_mints_account_info.realloc(next_allowed_mints_size, false)?;
        let rent = Rent::get()?;
        let next_rent_exemption = rent.minimum_balance(next_allowed_mints_size);
        if allowed_mints_account_info.lamports() < next_rent_exemption {
            let required_lamports = next_rent_exemption - allowed_mints_account_info.lamports();
            let transfer_rent = system_instruction::transfer(
                ctx.accounts.admin.key,
                &ctx.accounts.allowed_mints.key(),
                required_lamports,
            );
            anchor_lang::solana_program::program::invoke(
                &transfer_rent,
                &[
                    ctx.accounts.admin.to_account_info(),
                    ctx.accounts.allowed_mints.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }
    }

    ctx.accounts.allowed_mints.mints = allowed_mints;
    Ok(())
}

