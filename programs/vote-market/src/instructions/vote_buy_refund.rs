use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{VoteMarketConfig, VoteBuy};

#[derive(Accounts)]
#[instruction(epoch: u32)]
pub struct VoteBuyRefund<'info> {
    pub buyer: Signer<'info>,
    #[account(mut,
    associated_token::mint = mint,
    associated_token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut,
    associated_token::mint = mint,
    associated_token::authority = vote_buy
    )]
    pub token_vault: Account<'info, TokenAccount>,
    #[account(
    mut,
    has_one = mint,
    has_one = buyer,
    seeds = [
    b"vote-buy".as_ref(),
    epoch.to_le_bytes().as_ref(),
    config.key().as_ref(),
    gauge.key().as_ref()],
    bump
    )]
    pub vote_buy: Account<'info, VoteBuy>,
    pub mint: Account<'info, Mint>,
    #[account(has_one = gaugemeister)]
    pub config: Account<'info, VoteMarketConfig>,
    pub gauge: Account<'info, gauge_state::Gauge>,
    pub gaugemeister: Account<'info, gauge_state::Gaugemeister>,
    pub token_program: Program<'info, Token>,
}

pub fn vote_buy_refund(ctx: Context<VoteBuyRefund>, epoch: u32) -> Result<()> {
    use anchor_spl::token::spl_token;
    use anchor_lang::solana_program::program::invoke_signed;

    msg!(
        "Epoch: {} Current Rewards epoch {}",
        epoch,
        ctx.accounts.gaugemeister.current_rewards_epoch
    );
    let mut refund_amount = ctx.accounts.token_vault.amount;
    if epoch < ctx.accounts.gaugemeister.current_rewards_epoch {
        msg!("Claiming refund for expired claims");
    } else {
        msg!("Claiming refund for excess buy value");
        if let Some(max_amount) = ctx.accounts.vote_buy.max_amount {
            refund_amount = ctx
                .accounts
                .vote_buy
                .amount
                .checked_sub(max_amount)
                .ok_or(crate::errors::VoteMarketError::InvalidRefund)?;
            ctx.accounts.vote_buy.amount -= refund_amount;
        } else {
            return err!(crate::errors::VoteMarketError::MaxVoteBuyAmountNotSet);
        }
    }
    let transfer_ix = spl_token::instruction::transfer(
        &ctx.accounts.token_program.key(),
        &ctx.accounts.token_vault.key(),
        &ctx.accounts.buyer_token_account.key(),
        &ctx.accounts.vote_buy.key(),
        &[],
        refund_amount,
    )?;
    let (_, bump) = Pubkey::find_program_address(
        &[
            b"vote-buy".as_ref(),
            epoch.to_le_bytes().as_ref(),
            ctx.accounts.config.key().as_ref(),
            ctx.accounts.gauge.key().as_ref(),
        ],
        ctx.program_id,
    );
    invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.token_vault.to_account_info(),
            ctx.accounts.buyer_token_account.to_account_info(),
            ctx.accounts.vote_buy.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        &[&[
            b"vote-buy".as_ref(),
            epoch.to_le_bytes().as_ref(),
            ctx.accounts.config.key().as_ref(),
            ctx.accounts.gauge.key().as_ref(),
            &[bump],
        ]],
    )?;
    Ok(())
}

