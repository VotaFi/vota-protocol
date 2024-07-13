use crate::actions::rpc_retry::retry_rpc;
use helius::types::{Cluster, CreateSmartTransactionConfig, SmartTransactionConfig};
use helius::Helius;
use retry::delay::Fixed;
use retry::Error as RetryError;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig};
use solana_program::instruction::Instruction;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use std::env;
use solana_program::address_lookup_table::AddressLookupTableAccount;
use crate::errors::VoteMarketManagerError::SimulationFailed;

pub fn retry_logic<'a>(
    client: &'a RpcClient,
    payer: &'a Keypair,
    ixs: &'a mut Vec<Instruction>,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let debug = true;
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api_key: &str = &*env::var("HELIUS_KEY").unwrap();
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster).unwrap();
    if debug {
        let mut sim_tx = Transaction::new_with_payer(&ixs, Some(&payer.pubkey()));


        let sim_strategy = Fixed::from_millis(1000).take(5);
        let sim_result = retry::retry(sim_strategy, || {
            let latest_blockhash = retry_rpc(|| {
                client.get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
            });
            match latest_blockhash {
                Ok((blockhash, _)) => sim_tx.sign(&[payer], blockhash),
                Err(e) => return Err(RetryError {
                    tries: 0,
                    total_delay: std::time::Duration::from_millis(0),
                    error: "RPC failed to get latest blockhash",
                }),
            }
            let sim = retry_rpc(|| {
                client.simulate_transaction_with_config(&sim_tx, {
                    RpcSimulateTransactionConfig {
                        replace_recent_blockhash: false,
                        sig_verify: true,
                        commitment: Some(CommitmentConfig::confirmed()),
                        ..RpcSimulateTransactionConfig::default()
                    }
                })
            });
            return match sim {
                Ok(sim) => {
                    println!("simulated: {:?}", sim);
                    if sim.value.err.is_some() {
                        println!("Internal error simulating transaction, retry");
                        return Err(RetryError {
                            tries: 0,
                            total_delay: std::time::Duration::from_millis(0),
                            error: "RPC failed to simulate transaction",
                        });
                    }
                    Ok(sim)
                }
                Err(e) => {
                    println!("Error restult simulating transaction, retry");
                    Err(RetryError {
                        tries: 0,
                        total_delay: std::time::Duration::from_millis(0),
                        error: "RPC failed to simulate transaction",
                    })
                }
            }
        });
        match sim_result {
            Ok(sim) => {
                println!("simulated: {:?}", sim);
                if sim.value.err.is_some() {
                    println!("SIM ERROR: {:?}", sim.value.err);
                    return Err(Box::new(SimulationFailed { sim_info: sim.value.logs.unwrap().join(" ") }));
                }
            }
            Err(e) => {
                //TODO: MAke a proper error
                println!("Error simulating transaction: {:?}", e);
            }
        }
    }
    let mut tries = 0;
    loop {
        let result = rt.block_on(helius.send_smart_transaction(SmartTransactionConfig {
            create_config: CreateSmartTransactionConfig {
                instructions: ixs.clone(),
                signers: vec![payer],
                lookup_tables: None,
                fee_payer: Some(payer),
            },
            send_options: RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentLevel::Confirmed),
                encoding: None,
                max_retries: Some(5),
                min_context_slot: None,
            },
        }));
        match result {
            Ok(sig) => return Ok(sig),
            Err(e) => {
                if tries == 10 {
                    println!("Error sending transaction: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        }
        tries += 1;
        println!("Retrying transaction {}", tries);
    }
}
