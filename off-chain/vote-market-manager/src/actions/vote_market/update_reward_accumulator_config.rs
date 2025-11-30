// filepath: \\off-chain\\vote-market-manager\\src\\actions\\vote_market\\update_reward_accumulator_config.rs
use crate::actions::retry_logic;
use anchor_client::Client;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};

pub(crate) fn update_reward_accumulator_config(
    client: &RpcClient,
    anchor_client: &Client<&Keypair>,
    payer: &Keypair,
    config: Pubkey,
    reward_accumulator_program: Pubkey,
    namespace: [u8; 8],
) -> Result<(), Box<dyn std::error::Error>> {
    let program = anchor_client.program(vote_market::id()).unwrap();
    let (reward_accumulator_config_address, _bump) = Pubkey::find_program_address(
        &[b"reward-accumulator-config", config.as_ref()],
        &vote_market::id(),
    );

    let mut ixs = program
        .request()
        .signer(payer)
        .args(vote_market::instruction::UpdateRewardAccumulatorConfig {
            reward_accumulator_program,
            namespace,
        })
        .accounts(vote_market::accounts::CreateRewardAccumulatorConfig {
            config,
            reward_accumulator_config: reward_accumulator_config_address,
            admin: payer.pubkey(),
            system_program: solana_program::system_program::id(),
        })
        .instructions()
        .unwrap();

    println!("got here");
    let result = retry_logic::retry_logic_direct(client, payer, &mut ixs);

    match result {
        Ok(sig) => println!("Reward accumulator config updated: {:?}", sig),
        Err(e) => println!("Error updating reward accumulator config: {:?}", e),
    }
    Ok(())
}

