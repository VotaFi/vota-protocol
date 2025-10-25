use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{VoteMarketConfig, VoteBuy, AllowedMints};

#[derive(Accounts)]
#[instruction(epoch: u32)]
pub struct IncreaseVoteBuy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut,
    associated_token::mint = mint,
    associated_token::authority = buyer,
    )]
    pub buyer_token_account: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed,
    payer = buyer,
    associated_token::mint = mint,
    associated_token::authority = vote_buy
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,
    pub mint: Account<'info, Mint>,
    #[account(has_one = gaugemeister)]
    pub config: Box<Account<'info, VoteMarketConfig>>,
    pub gaugemeister: Account<'info, gauge_state::Gaugemeister>,
    #[account(init_if_needed,
    payer = buyer,
    space = VoteBuy::LEN,
    seeds = [b"vote-buy".as_ref(),
    epoch.to_le_bytes().as_ref(),
    config.key().as_ref(),
    gauge.key().as_ref()],
    bump)]
    pub vote_buy: Box<Account<'info, VoteBuy>>,
    #[account(has_one = gaugemeister, constraint = !gauge.is_disabled)]
    pub gauge: Account<'info, gauge_state::Gauge>,
    #[account(seeds = [b"allow-list".as_ref(), config.key().as_ref()], bump)]
    pub allowed_mints: Box<Account<'info, AllowedMints>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn increase_vote_buy(ctx: Context<IncreaseVoteBuy>, epoch: u32, amount: u64) -> Result<()> {
    use anchor_spl::token::spl_token;
    use anchor_lang::solana_program::program::invoke;

    if amount == 0 {
        return err!(crate::errors::VoteMarketError::InvalidVoteBuyAmount);
    }
    //Check buyer and mint
    if ctx.accounts.buyer.key() == Pubkey::default() {
        return err!(crate::errors::VoteMarketError::InvalidBuyer);
    }
    if ctx.accounts.mint.key() == Pubkey::default() {
        return err!(crate::errors::VoteMarketError::InvalidMint);
    }
    if ctx.accounts.vote_buy.buyer == Pubkey::default()
        && ctx.accounts.vote_buy.mint == Pubkey::default()
    {
        ctx.accounts.vote_buy.mint = ctx.accounts.mint.key();
        ctx.accounts.vote_buy.buyer = ctx.accounts.buyer.key();
        ctx.accounts.vote_buy.total_committed = 0;
    } else {
        // Only change the claimer if they increase the vote buy by more than 100%
        if amount > ctx.accounts.vote_buy.amount {
            msg!("setting new buyer");
            ctx.accounts.vote_buy.buyer = ctx.accounts.buyer.key();
        }
    }
    if ctx.accounts.vote_buy.mint != ctx.accounts.mint.key() {
        return err!(crate::errors::VoteMarketError::InvalidMint);
    }
    // Check epoch
    if ctx.accounts.gaugemeister.current_rewards_epoch + 1 > epoch {
        return err!(crate::errors::VoteMarketError::CompletedEpoch);
    }
    // Check if mint is valid
    ctx.accounts
        .allowed_mints
        .mints
        .iter()
        .find(|mint| mint == &&ctx.accounts.mint.key())
        .ok_or::<Error>(crate::errors::VoteMarketError::InvalidMint.into())?;
    let transfer_ix = spl_token::instruction::transfer(
        &ctx.accounts.token_program.key(),
        &ctx.accounts.buyer_token_account.key(),
        &ctx.accounts.token_vault.key(),
        &ctx.accounts.buyer.key(),
        &[],
        amount,
    )?;
    invoke(
        &transfer_ix,
        &[
            ctx.accounts.buyer_token_account.to_account_info(),
            ctx.accounts.token_vault.to_account_info(),
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;
    ctx.accounts.vote_buy.amount += amount;
    ctx.accounts.vote_buy.gauge = ctx.accounts.gauge.key();
    Ok(())
}

