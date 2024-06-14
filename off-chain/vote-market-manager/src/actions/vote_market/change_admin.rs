use crate::actions::retry_logic;
use anchor_client::Client;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

pub(crate) fn change_admin(
    client: &RpcClient,
    anchor_client: &Client<&Keypair>,
    payer: &Keypair,
    config: Pubkey,
    new_admin: Pubkey,
) -> Result<(), Box<dyn std::error::Error>> {
    let program = anchor_client.program(vote_market::id()).unwrap();
    let mut ixs = program
        .request()
        .signer(payer)
        .args(vote_market::instruction::UpdateAdmin { admin: new_admin })
        .accounts(vote_market::accounts::UpdateAdmin {
            config,
            admin: payer.pubkey(),
        })
        .instructions()
        .unwrap();
    let result = retry_logic::retry_logic(client, payer, &mut ixs, None);

    match result {
        Ok(sig) => println!("Admin updated: {:?}", sig),
        Err(e) => println!("Error updating admin: {:?}", e),
    }
    Ok(())
}
