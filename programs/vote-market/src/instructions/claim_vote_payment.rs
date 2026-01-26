use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use locked_voter_state::LockedVoterProgram;
use gauge_state::GaugeProgram;
use crate::state::{VoteMarketConfig, VoteBuy};
use std::cmp::min;

#[derive(Accounts)]
#[instruction(epoch: u32)]
pub struct ClaimVotePayment<'info> {
    pub script_authority: Signer<'info>,
    #[account(mut)]
    pub seller: SystemAccount<'info>,
    #[account(mut,
    associated_token::mint = mint,
    associated_token::authority = seller,
    )]
    pub seller_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut,
    associated_token::mint = mint,
    associated_token::authority = vote_buy,
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut,
    associated_token::mint = mint,
    associated_token::authority = admin,
    )]
    /// CHECK Checked by seed constraints
    pub treasury: Box<Account<'info, TokenAccount>>,
    /// CHECK Not enough stack space to deserialize. Only used to check treasury seeds.
    pub admin: UncheckedAccount<'info>,
    pub mint: Account<'info, Mint>,
    #[account(has_one = gaugemeister, has_one = script_authority, has_one = admin)]
    pub config: Box<Account<'info, VoteMarketConfig>>,
    #[account(mut,
    seeds = [b"vote-buy".as_ref(),
    epoch.to_le_bytes().as_ref(),
    config.key().as_ref(),
    gauge.key().as_ref()], bump)]
    pub vote_buy: Box<Account<'info, VoteBuy>>,
    #[account(mut, seeds = [b"vote-delegate", config.key().as_ref()], bump)]
    pub vote_delegate: SystemAccount<'info>,
    #[account(has_one = vote_delegate,
    constraint = escrow.owner == seller.key(),
    owner = locked_voter_program.key(),
    seeds = [b"Escrow",
        gaugemeister.locker.as_ref(),
        escrow.owner.as_ref()],
    bump,
    seeds::program = locked_voter_state::id())]
    pub escrow: Account<'info, locked_voter_state::Escrow>,
    #[account(owner = gauge_program.key(),
    constraint = gaugemeister.locker == escrow.locker)]
    pub gaugemeister: Account<'info, gauge_state::Gaugemeister>,
    #[account(has_one = gaugemeister,
    has_one = escrow,
    seeds=[b"GaugeVoter",
    gaugemeister.key().as_ref(),
    escrow.key().as_ref()], bump,
    seeds::program = gauge_program.key(),
    )]
    pub gauge_voter: Account<'info, gauge_state::GaugeVoter>,
    #[account(has_one = gauge_voter,
    has_one = gauge,
    seeds=[b"GaugeVote",
    gauge_voter.key().as_ref(),
    gauge.key().as_ref()],
    bump,
    seeds::program = gauge_program.key()
    )]
    pub gauge_vote: Account<'info, gauge_state::GaugeVote>,
    #[account(has_one = gauge_voter, owner = gauge_program.key(),
    seeds=[b"EpochGaugeVoter",
    gauge_voter.key().as_ref(),
    epoch.to_le_bytes().as_ref()],
    bump,
    seeds::program = gauge_program.key(),
    )]
    pub epoch_gauge_voter: Account<'info, gauge_state::EpochGaugeVoter>,
    #[account(has_one = gaugemeister, constraint = !gauge.is_disabled)]
    pub gauge: Account<'info, gauge_state::Gauge>,
    #[account(has_one = gauge, owner = gauge_program.key())]
    // Seeds checked in instruction body
    pub epoch_gauge: Account<'info, gauge_state::EpochGauge>,
    #[account(mut, owner = gauge_program.key())]
    // Seeds checked in instruction body
    pub epoch_gauge_vote: Account<'info, gauge_state::EpochGaugeVote>,
    pub gauge_program: Program<'info, GaugeProgram>,
    pub locked_voter_program: Program<'info, LockedVoterProgram>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn claim_vote_payment(ctx: Context<ClaimVotePayment>, epoch: u32) -> Result<()> {
    use anchor_spl::token::spl_token;
    use anchor_lang::solana_program::program::{invoke_signed};
    use anchor_lang::solana_program;
    use crate::util::vote_math::{get_fee, get_user_payment};

    //seed checks. Doing this on the Accounts struct uses too much stack space
    msg!("Claiming payment");
    let (expected_epoch_gauge, _) = Pubkey::find_program_address(
        &[
            b"EpochGauge".as_ref(),
            ctx.accounts.gauge.key().as_ref(),
            epoch.to_le_bytes().as_ref(),
        ],
        &gauge_state::id(),
    );
    require_keys_eq!(expected_epoch_gauge, ctx.accounts.epoch_gauge.key());
    let (expected_epoch_guage_vote, _) = Pubkey::find_program_address(
        &[
            b"EpochGaugeVote".as_ref(),
            ctx.accounts.gauge_vote.key().as_ref(),
            epoch.to_le_bytes().as_ref(),
        ],
        &gauge_state::id(),
    );
    require_keys_eq!(
        expected_epoch_guage_vote,
        ctx.accounts.epoch_gauge_vote.key()
    );
    if epoch > ctx.accounts.gaugemeister.current_rewards_epoch {
        return err!(crate::errors::VoteMarketError::EpochVotingNotCompleted);
    }
    let total_power = u64::min(ctx.accounts.vote_buy.total_committed, ctx.accounts.epoch_gauge.total_power);
    let allocated_power = ctx.accounts.epoch_gauge_vote.allocated_power;

    let vote_buy = &ctx.accounts.vote_buy;
    let total_vote_payment = match vote_buy.max_amount {
        Some(max_amount) => min(max_amount, vote_buy.amount),
        None => {
            return err!(crate::errors::VoteMarketError::MaxVoteBuyAmountNotSet);
        }
    };
    msg!("Total Power: {}", total_power);
    msg!("Allocated Power: {}", allocated_power);
    msg!("Total Vote Payment: {}", total_vote_payment);
    let total_payment = get_user_payment(total_power, allocated_power, total_vote_payment)?;
    let fee = get_fee(total_payment, ctx.accounts.config.claim_fee)?;
    let payment_to_user = total_payment - fee;
    let transfer_ix = spl_token::instruction::transfer(
        &ctx.accounts.token_program.key(),
        &ctx.accounts.token_vault.key(),
        &ctx.accounts.seller_token_account.key(),
        &vote_buy.key(),
        &[],
        payment_to_user,
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
            ctx.accounts.seller_token_account.to_account_info(),
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
    if fee != 0 {
        let transfer_to_treasury_ix = spl_token::instruction::transfer(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.token_vault.key(),
            &ctx.accounts.treasury.key(),
            &vote_buy.key(),
            &[],
            fee,
        )?;
        invoke_signed(
            &transfer_to_treasury_ix,
            &[
                ctx.accounts.token_vault.to_account_info(),
                ctx.accounts.treasury.to_account_info(),
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
    }

    //Calculating the discriminator manually instead of including the crate
    //because the anchor_lang version of gauge is not compatible with this program.
    let mut data: Vec<u8> =
        solana_program::hash::hash(b"global:close_epoch_gauge_vote").to_bytes()[..8].to_vec();
    data.extend_from_slice(&epoch.to_le_bytes());
    let (_, vote_delegate_bump) = Pubkey::find_program_address(
        &[
            b"vote-delegate".as_ref(),
            ctx.accounts.config.key().as_ref(),
        ],
        ctx.program_id,
    );
    let close_ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.accounts.gauge_program.key(),
        accounts: vec![
            AccountMeta::new(ctx.accounts.epoch_gauge_vote.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gaugemeister.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge_voter.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge_vote.key(), false),
            AccountMeta::new_readonly(ctx.accounts.escrow.key(), false),
            AccountMeta::new_readonly(ctx.accounts.vote_delegate.key(), true),
            AccountMeta::new(ctx.accounts.vote_delegate.key(), false),
        ],
        data,
    };
    invoke_signed(
        &close_ix,
        &[
            ctx.accounts.epoch_gauge_vote.to_account_info(),
            ctx.accounts.gaugemeister.to_account_info(),
            ctx.accounts.gauge.to_account_info(),
            ctx.accounts.gauge_voter.to_account_info(),
            ctx.accounts.gauge_vote.to_account_info(),
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.vote_delegate.to_account_info(),
            ctx.accounts.vote_delegate.to_account_info(),
        ],
        &[&[
            b"vote-delegate".as_ref(),
            ctx.accounts.config.key().as_ref(),
            &[vote_delegate_bump],
        ]],
    )?;
    Ok(())
}

