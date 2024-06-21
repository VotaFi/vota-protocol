use crate::actions::retry_logic;
use anchor_client::Client;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

pub(crate) fn change_script_authority(
    client: &RpcClient,
    anchor_client: &Client<&Keypair>,
    payer: &Keypair,
    config: Pubkey,
    new_script_authority: Pubkey,
) -> Result<(), Box<dyn std::error::Error>> {
    let program = anchor_client.program(vote_market::id()).unwrap();
    let mut ixs = program
        .request()
        .signer(payer)
        .args(vote_market::instruction::UpdateScriptAuthority {
            script_authority: new_script_authority,
        })
        .accounts(vote_market::accounts::UpdateScriptAuthority {
            config,
            admin: payer.pubkey(),
        })
        .instructions()
        .unwrap();
    let result = retry_logic::retry_logic(client, payer, &mut ixs);

    match result {
        Ok(sig) => println!("Script authority updated: {:?}", sig),
        Err(e) => println!("Error updating script authority: {:?}", e),
    }
    Ok(())
}
