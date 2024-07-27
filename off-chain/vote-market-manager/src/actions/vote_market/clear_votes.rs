use crate::accounts::resolve::{get_delegate, get_gauge_vote, get_gauge_voter};
use crate::actions::management::utils;
use crate::actions::retry_logic::{retry_logic_helius};
use crate::{GAUGEMEISTER, LOCKER};
use anchor_client::Client;
use anchor_lang::AnchorDeserialize;
use helius::Helius;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

pub(crate) async fn clear_votes(
    anchor_client: &Client<&Keypair>,
    helius: &Helius,
    script_authority: &Keypair,
    config: Pubkey,
    owner: Pubkey,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("clearing votes");
    let program = anchor_client.program(vote_market::id())?;
    let gauges = utils::get_relevant_gauges()?;
    let vote_delegate = get_delegate(&config);
    let (escrow, _) = Pubkey::find_program_address(
        &[
            b"Escrow",
            LOCKER.to_bytes().as_ref(),
            owner.to_bytes().as_ref(),
        ],
        &locked_voter_state::id(),
    );
    let gauge_voter = get_gauge_voter(&escrow);
    let gauge_votes = gauges
        .iter()
        .map(|g| get_gauge_vote(&get_gauge_voter(&escrow), g))
        .collect::<Vec<Pubkey>>();
    let gauge_vote_accounts = helius.async_connection()?.get_multiple_accounts(&gauge_votes).await?;

    for (i, gauge) in gauges.iter().enumerate() {
        // Can only clear initialized gauge_votes
        if gauge_vote_accounts[i].is_none() {
            println!("not found");
            continue;
        }
        let gauge_data = gauge_state::GaugeVote::deserialize(
            &mut &gauge_vote_accounts[i].clone().unwrap().data[8..],
        )?;
        if gauge_data.weight == 0 {
            continue;
        }
        let gauge_vote = gauge_votes[i];
        let mut vote_ixs = program
            .request()
            .signer(script_authority)
            .args(vote_market::instruction::Vote { weight: 0 })
            .accounts(vote_market::accounts::Vote {
                config,
                script_authority: script_authority.pubkey(),
                gaugemeister: GAUGEMEISTER,
                gauge: *gauge,
                gauge_voter: gauge_voter,
                gauge_vote: gauge_vote,
                escrow,
                vote_delegate,
                gauge_program: gauge_state::id(),
            })
            .instructions()
            .unwrap();
        println!("Clearing votes");
        let result = retry_logic_helius(helius, script_authority, &mut vote_ixs).await;
        match result {
            Ok(sig) => {
                log::info!(target: "vote",
                sig=sig.to_string(),
                user=owner.to_string(),
                config=config.to_string();
                "cleared votes");
                println!("Cleared votes for {:?}: {:?}", escrow, sig);
            }
            Err(e) => {
                log::error!(target: "vote",
                error=e.to_string(),
                user=owner.to_string(),
                config=config.to_string();
                "failed to clear votes");
                println!("Error clearing votes for {:?}: {:?}", escrow, e);
            }
        }
    }
    println!("cleared votes!");
    Ok(())
}
