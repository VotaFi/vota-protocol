use anchor_lang::prelude::*;
use gauge_state::GaugeProgram;
use crate::state::{VoteMarketConfig, VoteBuy};

#[derive(Accounts)]
#[instruction(epoch: u32)]
pub struct RevertVote<'info> {
    #[account(has_one = gaugemeister, has_one = script_authority)]
    pub config: Account<'info, VoteMarketConfig>,
    #[account(mut)]
    pub script_authority: Signer<'info>,
    #[account(owner = gauge_state::id())]
    pub gaugemeister: Account<'info, gauge_state::Gaugemeister>,
    #[account(has_one = gaugemeister)]
    pub gauge: Account<'info, gauge_state::Gauge>,
    pub gauge_voter: Account<'info, gauge_state::GaugeVoter>,
    #[account(
    seeds=[b"GaugeVote",
    gauge_voter.key().as_ref(),
    gauge.key().as_ref()],
    bump,
    seeds::program = gauge_state::id(),
    )]
    pub gauge_vote: Account<'info, gauge_state::GaugeVote>,
    #[account(mut)]
    pub epoch_gauge: Account<'info, gauge_state::EpochGauge>,
    #[account(mut,
    has_one = gauge_voter, owner = gauge_state::id(),
    seeds=[b"EpochGaugeVoter",
    gauge_voter.key().as_ref(),
    epoch.to_le_bytes().as_ref()],
    bump,
    seeds::program = gauge_state::id(),
    )]
    pub epoch_gauge_voter: Account<'info, gauge_state::EpochGaugeVoter>,
    /// CHECK: Verified by the gauge program via the gauge_voter has_one.
    pub escrow: UncheckedAccount<'info>,
    #[account(mut,
    seeds=[b"EpochGaugeVote",
    gauge_vote.key().as_ref(),
    epoch_gauge_voter.voting_epoch.to_le_bytes().as_ref()],
    bump,
    seeds::program = gauge_state::id(),
    )]
    /// CHECK This will be closed through a CPI
    pub epoch_gauge_vote: UncheckedAccount<'info>,
    #[account(mut,
    seeds=[b"vote-buy",
    epoch.to_le_bytes().as_ref(),
    config.key().as_ref(),
    gauge.key().as_ref()],
    bump
    )]
    pub vote_buy: Account<'info, VoteBuy>,
    #[account(mut,
    seeds =
    [b"vote-delegate", config.key().as_ref()],
    bump)]
    pub vote_delegate: SystemAccount<'info>,
    pub gauge_program: Program<'info, GaugeProgram>,
    pub system_program: Program<'info, System>,
}

/// Reverts a previously committed vote for the given epoch.
///
/// This is the inverse of [`commit_vote`]. It invokes the gauge program's
/// `gauge_revert_vote` instruction, signed by the program's vote-delegate PDA.
#[allow(unused_variables)]
pub fn revert_vote(ctx: Context<RevertVote>, epoch: u32) -> Result<()> {
    use anchor_lang::solana_program::program::invoke_signed;
    use anchor_lang::solana_program;

    let data: Vec<u8> =
        solana_program::hash::hash(b"global:gauge_revert_vote").to_bytes()[..8].to_vec();
    let revert_vote_ix = solana_program::instruction::Instruction {
        program_id: gauge_state::id(),
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.gaugemeister.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge_voter.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge_vote.key(), false),
            AccountMeta::new(ctx.accounts.epoch_gauge.key(), false),
            AccountMeta::new(ctx.accounts.epoch_gauge_voter.key(), false),
            AccountMeta::new_readonly(ctx.accounts.escrow.key(), false),
            AccountMeta::new_readonly(ctx.accounts.vote_delegate.key(), true),
            AccountMeta::new(ctx.accounts.epoch_gauge_vote.key(), false),
            AccountMeta::new(ctx.accounts.script_authority.key(), true),
        ],
        data,
    };
    let (expected_vote_delegate, bump) = Pubkey::find_program_address(
        &[
            b"vote-delegate".as_ref(),
            ctx.accounts.config.key().as_ref(),
        ],
        ctx.program_id,
    );
    require_keys_eq!(expected_vote_delegate, ctx.accounts.vote_delegate.key());
    invoke_signed(
        &revert_vote_ix,
        &[
            ctx.accounts.gaugemeister.to_account_info(),
            ctx.accounts.gauge.to_account_info(),
            ctx.accounts.gauge_voter.to_account_info(),
            ctx.accounts.gauge_vote.to_account_info(),
            ctx.accounts.epoch_gauge.to_account_info(),
            ctx.accounts.epoch_gauge_voter.to_account_info(),
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.vote_delegate.to_account_info(),
            ctx.accounts.epoch_gauge_vote.to_account_info(),
            ctx.accounts.script_authority.to_account_info(),
        ],
        &[&[
            b"vote-delegate".as_ref(),
            ctx.accounts.config.key().as_ref(),
            &[bump],
        ]],
    )?;

    // Mirror commit_vote: subtract back the voting power that was previously committed
    // for this user, so that vote_buy.total_committed reflects the current state.
    let power: u64 = ctx.accounts.epoch_gauge_voter.voting_power;
    ctx.accounts.vote_buy.total_committed = ctx
        .accounts
        .vote_buy
        .total_committed
        .saturating_sub(power);
    Ok(())
}

