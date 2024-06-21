use crate::actions::rpc_retry::retry_rpc;
use helius::types::{Cluster, SmartTransactionConfig};
use helius::Helius;
use retry::delay::{Fixed};
use retry::{Error as RetryError};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig};
use solana_program::instruction::Instruction;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use std::env;

pub fn retry_logic<'a>(
    client: &'a RpcClient,
    payer: &'a Keypair,
    ixs: &'a mut Vec<Instruction>,
) -> helius::error::Result<Signature> {
    let debug = true;
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api_key: &str = &*env::var("HELIUS_KEY").unwrap();
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster).unwrap();
    if debug {
        let sim_strategy = Fixed::from_millis(200).take(5);

        let mut sim_tx = Transaction::new_with_payer(&ixs, Some(&payer.pubkey()));

        let (latest_blockhash, _) = retry_rpc(|| {
            client.get_latest_blockhash_with_commitment({
                CommitmentConfig {
                    commitment: CommitmentLevel::Confirmed,
                }
            })
        })
            .or_else(|_| {
                Err(RetryError {
                    tries: 0,
                    total_delay: std::time::Duration::from_millis(0),
                    error: "RPC failed to get blockhash",
                })
            }).unwrap();
        sim_tx.sign(&[payer], latest_blockhash);

        let sim_result = retry::retry(sim_strategy, || {
            let sim = retry_rpc(|| {
                client.simulate_transaction_with_config(&sim_tx, {
                    RpcSimulateTransactionConfig {
                        replace_recent_blockhash: false,
                        sig_verify: true,
                        commitment: Some(CommitmentConfig {
                            commitment: CommitmentLevel::Confirmed,
                        }),
                        ..RpcSimulateTransactionConfig::default()
                    }
                })
            })
                .or_else(|e| {
                    println!("Error simulating transaction: {:?}", e);
                    Err(RetryError {
                        tries: 0,
                        total_delay: std::time::Duration::from_millis(0),
                        error: "RPC failed to simulate transaction",
                    })
                });
            sim
        });
        match sim_result {
            Ok(sim) => {
                println!("simulated: {:?}", sim);
            }
            Err(e) => {
                //TODO: MAke a proper error
                println!("Error simulating transaction: {:?}", e);
            }
        }
    }
    let mut tries = 0;
    let mut result;
    loop {
        result = rt.block_on(helius.send_smart_transaction(SmartTransactionConfig {
            instructions: ixs.clone(),
            signers: vec![payer],
            send_options: RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: None,
                encoding: None,
                max_retries: None,
                min_context_slot: None,
            },
            lookup_tables: None,
        }));
        if result.is_ok() || tries == 10 {
            return result;
        }
        tries += 1;
        println!("Retrying transaction {}", tries);
    }
}
