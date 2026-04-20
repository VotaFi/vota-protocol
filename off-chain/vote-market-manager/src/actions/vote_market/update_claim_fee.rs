use crate::actions::retry_logic;
use anchor_client::Client;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

pub(crate) fn update_claim_fee(
    client: &RpcClient,
    anchor_client: &Client<&Keypair>,
    payer: &Keypair,
    config: Pubkey,
    claim_fee: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let program = anchor_client.program(vote_market::id()).unwrap();
    let mut ixs = program
        .request()
        .signer(payer)
        .args(vote_market::instruction::UpdateClaimFee { claim_fee })
        .accounts(vote_market::accounts::UpdateClaimFee {
            config,
            admin: payer.pubkey(),
        })
        .instructions()
        .unwrap();
    let result = retry_logic::retry_logic_direct(client, payer, &mut ixs);

    match result {
        Ok(sig) => println!("Claim fee updated to {}: {:?}", claim_fee, sig),
        Err(e) => println!("Error updating claim fee: {:?}", e),
    }
    Ok(())
}

