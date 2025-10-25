use anchor_lang::prelude::*;
use gauge_state::GaugeProgram;
use crate::state::{VoteMarketConfig, VoteBuy};

#[derive(Accounts)]
#[instruction(epoch: u32)]
pub struct CommitVote<'info> {
    #[account(has_one = gaugemeister, has_one = script_authority)]
    pub config: Account<'info, VoteMarketConfig>,
    #[account(mut)]
    pub script_authority: Signer<'info>,
    #[account(owner = gauge_state::id())]
    pub gaugemeister: Account<'info, gauge_state::Gaugemeister>,
    #[account(has_one = gaugemeister, constraint = !gauge.is_disabled)]
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
    #[account(mut,
    seeds=[b"EpochGaugeVote",
    gauge_vote.key().as_ref(),
    epoch_gauge_voter.voting_epoch.to_le_bytes().as_ref()],
    bump,
    seeds::program = gauge_state::id(),
    )]
    /// CHECK This will be initialized through a CPI
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

/// This is used in a trait on the Accounts definition
#[allow(unused_variables)]
pub fn commit_vote(ctx: Context<CommitVote>, epoch: u32) -> Result<()> {
    use anchor_lang::solana_program::program::invoke_signed;
    use anchor_lang::solana_program;

    let data: Vec<u8> =
        solana_program::hash::hash(b"global:gauge_commit_vote_v2").to_bytes()[..8].to_vec();
    let set_weight_ix = solana_program::instruction::Instruction {
        program_id: gauge_state::id(),
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.gaugemeister.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge_voter.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge_vote.key(), false),
            AccountMeta::new(ctx.accounts.epoch_gauge.key(), false),
            AccountMeta::new(ctx.accounts.epoch_gauge_voter.key(), false),
            AccountMeta::new(ctx.accounts.epoch_gauge_vote.key(), false),
            AccountMeta::new(ctx.accounts.vote_delegate.key(), true),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
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
        &set_weight_ix,
        &[
            ctx.accounts.gaugemeister.to_account_info(),
            ctx.accounts.gauge.to_account_info(),
            ctx.accounts.gauge_voter.to_account_info(),
            ctx.accounts.gauge_vote.to_account_info(),
            ctx.accounts.epoch_gauge.to_account_info(),
            ctx.accounts.epoch_gauge_voter.to_account_info(),
            ctx.accounts.epoch_gauge_vote.to_account_info(),
            ctx.accounts.vote_delegate.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[&[
            b"vote-delegate".as_ref(),
            ctx.accounts.config.key().as_ref(),
            &[bump],
        ]],
    )?;

    let power: u64 = ctx.accounts.epoch_gauge_voter.voting_power;
    let total_shares = ::u128::mul_div_u64(
        power,
        ctx.accounts.gauge_vote.weight.into(),
        ctx.accounts.gauge_voter.total_weight.into(),
    ).ok_or(crate::errors::VoteMarketError::InvalidVotePower)?;
    ctx.accounts.vote_buy.total_committed += power;
    Ok(())
}

