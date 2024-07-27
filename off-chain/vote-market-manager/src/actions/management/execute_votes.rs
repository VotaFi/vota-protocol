use crate::actions::management::data::{EpochData, VoteInfo};
use crate::accounts::resolve::{
    get_epoch_gauge_voter, get_escrow_address_for_owner, get_gauge_voter,
};
use crate::actions::vote_market::clear_votes::clear_votes;
use crate::actions::vote_market::vote::vote;
use anchor_client::Client;
use helius::Helius;
use solana_sdk::signature::Keypair;

pub(crate) async fn execute_votes(
    helius: &Helius,
    anchor_client: &Client<&Keypair>,
    script_authority: &Keypair,
    data: EpochData,
    vote_weights: Vec<VoteInfo>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Executing votes");
    println!("Data: {:?}", data);
    println!("Vote weights: {:?}", vote_weights);
    for (i, escrow_owner) in data.escrow_owners.iter().enumerate() {
        println!(
            "Voting on behalf of escrow owner {:?}\n Escrow {} out of {}",
            escrow_owner,
            i,
            data.escrow_owners.len()
        );
        let escrow = get_escrow_address_for_owner(&escrow_owner);
        let gauge_voter = get_gauge_voter(&escrow);
        let epoch_gauge_voter = get_epoch_gauge_voter(&gauge_voter, data.epoch);
        println!("epoch_guage_voter {:?}", epoch_gauge_voter);
        let epoch_gauge_voter_account = helius.rpc_client.solana_client.get_account(&epoch_gauge_voter);
        // TODO: Actually need to check that all votes are committed.
        let mut skip_weights = false;
        println!("skip_weights: {:?}", skip_weights);
        if epoch_gauge_voter_account.is_ok() {
            println!("Epoch gauge voter found. Already voted");
            skip_weights = true;
            // println!("Epoch gauge voter found, resetting");
            // reset_epoch_gauge_voter(client, script_authority, *escrow_owner, data.epoch);
        }
        println!("skip_weights: {:?}", skip_weights);
        if !skip_weights {
            println!("going to clear votes");
            let cv = clear_votes(
                anchor_client,
                helius,
                script_authority,
                data.config,
                *escrow_owner,
            ).await;
            //delay for 5 seconds to allow for votes to clear
            println!("does it crash after this?");
            match cv {
                Ok(_) => println!("Votes cleared"),
                Err(e) => {
                    println!("Stop");
                },
            }
            //tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            println!("nope");
        }

        let result = vote(
            anchor_client,
            helius,
            script_authority,
            data.config,
            *escrow_owner,
            data.epoch,
            vote_weights.clone(),
            skip_weights,
        ).await;
        match result {
            Ok(_) => println!("Escrow owner: {:?} voted", escrow_owner),
            Err(e) => {
                log::error!(target: "vote",
                    error=e.to_string(),
                    user=escrow_owner.to_string(),
                    config=data.config.to_string(),
                    epoch=data.epoch;
                    "failed to set vote weight");
            },
        }
    }
    Ok(())
}
