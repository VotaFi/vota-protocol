use anchor_lang::prelude::*;
use gauge_state::GaugeProgram;
use crate::state::VoteMarketConfig;

#[derive(Accounts)]
#[instruction(weight: u32)]
pub struct Vote<'info> {
    #[account(has_one = gaugemeister, has_one = script_authority)]
    pub config: Account<'info, VoteMarketConfig>,
    pub script_authority: Signer<'info>,
    #[account(owner = gauge_state::id())]
    pub gaugemeister: Account<'info, gauge_state::Gaugemeister>,
    #[account(has_one = gaugemeister, constraint = !gauge.is_disabled)]
    pub gauge: Account<'info, gauge_state::Gauge>,
    #[account(mut,
    seeds=[b"GaugeVoter",
    gaugemeister.key().as_ref(),
    escrow.key().as_ref()], bump,
    seeds::program = gauge_state::id(),
    )]
    pub gauge_voter: Account<'info, gauge_state::GaugeVoter>,
    #[account(mut,
    seeds=[b"GaugeVote",
    gauge_voter.key().as_ref(),
    gauge.key().as_ref()],
    bump,
    seeds::program = gauge_state::id(),
    )]
    pub gauge_vote: Account<'info, gauge_state::GaugeVote>,
    #[account(has_one = vote_delegate,
    seeds = [b"Escrow",
    gaugemeister.locker.as_ref(),
    escrow.owner.as_ref()],
    bump,
    seeds::program = locked_voter_state::id())]
    pub escrow: Account<'info, locked_voter_state::Escrow>,
    #[account(mut, seeds =
    [b"vote-delegate", config.key().as_ref()],
    bump)]
    pub vote_delegate: SystemAccount<'info>,
    pub gauge_program: Program<'info, GaugeProgram>,
}

pub fn vote(ctx: Context<Vote>, weight: u32) -> Result<()> {
    use anchor_lang::solana_program::program::invoke_signed;
    use anchor_lang::solana_program;

    let mut data: Vec<u8> =
        solana_program::hash::hash(b"global:gauge_set_vote").to_bytes()[..8].to_vec();
    data.extend_from_slice(weight.to_le_bytes().as_ref());
    let set_weight_ix = solana_program::instruction::Instruction {
        program_id: gauge_state::id(),
        accounts: vec![
            AccountMeta::new_readonly(ctx.accounts.gaugemeister.key(), false),
            AccountMeta::new_readonly(ctx.accounts.gauge.key(), false),
            AccountMeta::new(ctx.accounts.gauge_voter.key(), false),
            AccountMeta::new(ctx.accounts.gauge_vote.key(), false),
            AccountMeta::new_readonly(ctx.accounts.escrow.key(), false),
            AccountMeta::new(ctx.accounts.vote_delegate.key(), true),
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
            ctx.accounts.escrow.to_account_info(),
            ctx.accounts.vote_delegate.to_account_info(),
        ],
        &[&[
            b"vote-delegate".as_ref(),
            ctx.accounts.config.key().as_ref(),
            &[bump],
        ]],
    )?;
    Ok(())
}

