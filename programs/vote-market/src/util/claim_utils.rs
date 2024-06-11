use anchor_lang::context::Context;
use anchor_lang::prelude::{AccountMeta,Pubkey};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::{ Key, solana_program, ToAccountInfo};
use crate::{ClaimVotePayment };

pub fn close_vote_account(ctx: &Context<ClaimVotePayment>, epoch: &u32) -> anchor_lang::Result<()> {
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
