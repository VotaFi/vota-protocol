use std::env;
use crate::actions::rpc_retry::retry_rpc;
use dotenv::Error;
use helius::Helius;
use helius::types::{Cluster, GetPriorityFeeEstimateRequest, SmartTransactionConfig};
use retry::delay::{Exponential, Fixed};
use retry::{Error as RetryError, OperationResult};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig};
use solana_client::rpc_response::RpcSimulateTransactionResult;
use solana_program::clock::Slot;
use solana_program::instruction::Instruction;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;

pub fn retry_logic<'a>(
    client: &'a RpcClient,
    payer: &'a Keypair,
    ixs: &'a mut Vec<Instruction>,
    max_cus: Option<u32>,
) -> helius::error::Result<Signature> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let api_key: &str = &*env::var("HELIUS_KEY").unwrap();
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster).unwrap();
    //    let jito_client = RpcClient::new("https://mainnet.block-engine.jito.wtf/api/v1/transactions");
    let result = rt.block_on(helius.send_smart_transaction(
        SmartTransactionConfig {
            instructions: ixs.clone(),
            signers: vec![payer],
            send_options: RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: None,
                encoding: None,
                max_retries: None,
                min_context_slot: None,
            },
            lookup_tables: None
    }));
    println!("Transaction result: {:?}", result);
    result
    // return match result {
    //     Ok(sig) => Ok(sig),
    //     Err(e) => Err(RetryError {
    //         tries: 0,
    //         total_delay: std::time::Duration::from_millis(0),
    //         error: "RPC failed to get sim results",
    //     }),
    // }

    // let sim_ixs = ixs.clone();
    // let mut sim_tx = Transaction::new_with_payer(&sim_ixs, Some(&payer.pubkey()));
    // let (latest_blockhash, _) = retry_rpc(|| {
    //     client.get_latest_blockhash_with_commitment({
    //         CommitmentConfig {
    //             commitment: CommitmentLevel::Confirmed,
    //         }
    //     })
    // })
    // .or_else(|_| {
    //     Err(RetryError {
    //         tries: 0,
    //         total_delay: std::time::Duration::from_millis(0),
    //         error: "RPC failed to get blockhash",
    //     })
    // })?;
    // sim_tx.sign(&[payer], latest_blockhash);
    // // From Helius discord
    // //I recommend following these best practices:
    // // * using alpha piriorty fee api from Helius to get a more reliable fee
    // // * sending transactions with maxRetries=0
    // // * polling transactions status with different commitment levels, and keep sending the same signed transaction (with maxRetries=0 and skipPreflight=true) until it gets to confirmed using exponential backoff
    // // * aborting if the blockheight goes over the lastValidBlockHeight
    // // delay for 1 sec to ensure blockhash is found by sim
    // std::thread::sleep(std::time::Duration::from_secs(4));
    // let sim_strategy = Fixed::from_millis(200).take(5);
    // let sim_result = retry::retry(sim_strategy, || {
    //     let sim = retry_rpc(|| {
    //         client.simulate_transaction_with_config(&sim_tx, {
    //             RpcSimulateTransactionConfig {
    //                 replace_recent_blockhash: false,
    //                 sig_verify: true,
    //                 commitment: Some(CommitmentConfig {
    //                     commitment: CommitmentLevel::Confirmed,
    //                 }),
    //                 ..RpcSimulateTransactionConfig::default()
    //             }
    //         })
    //     })
    //     .or_else(|_| {
    //         Err(RetryError {
    //             tries: 0,
    //             total_delay: std::time::Duration::from_millis(0),
    //             error: "RPC failed to simulate transaction",
    //         })
    //     });
    //     sim
    // });
    // let mut sim_cus: Option<u64> = None;
    // match sim_result {
    //     Ok(sim) => {
    //         println!("simulated: {:?}", sim);
    //         if sim.value.err.is_some() {
    //             return Err(RetryError {
    //                 tries: 0,
    //                 total_delay: std::time::Duration::from_millis(0),
    //                 error: "Simulated transaction returned an error",
    //             });
    //         }
    //         sim_cus = sim.value.units_consumed;
    //     }
    //     Err(e) => {
    //         //TODO: MAke a proper error
    //         println!("Error simulating transaction: {:?}", e);
    //         return Err(RetryError {
    //             tries: 0,
    //             total_delay: std::time::Duration::from_millis(0),
    //             error: "RPC failed to get sim results",
    //         });
    //     }
    // }
    // if let Some(cus) = sim_cus {
    //     let max_cus_ix =
    //         solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(
    //             (cus as u32) + 1000,
    //         );
    //     ixs.insert(0, max_cus_ix);
    //     let priority_fee_response = rt.block_on(helius.rpc_client.get_priority_fee_estimate(
    //         GetPriorityFeeEstimateRequest {
    //             transaction: None,
    //             account_keys: Some(vec!["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string()]),
    //             options: None,
    //         }
    //     )).unwrap();
    //     //
    //     let priority_fee = priority_fee_response.priority_fee_estimate.unwrap() + (1000 as f64);
    //     println!("Priority fee estimate: {}", priority_fee);
    //     println!("Max cus: {}", cus);
    //     let micro_lamports = (((priority_fee) / (cus as f64)) * 1_000_000.0) as u64;
    //     println!("Priority fee: {}", micro_lamports);
    //     let priority_fee_ix =
    //         solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(
    //             micro_lamports,
    //         );
    //     // Add the priority fee instruction to the beginning of the transaction
    //     ixs.insert(0, priority_fee_ix);
    // }
    //
    // let mut try_number = 0;
    //
    // loop {
    //     let (latest_blockhash, _) = retry_rpc(|| {
    //         client.get_latest_blockhash_with_commitment({
    //             CommitmentConfig {
    //                 commitment: CommitmentLevel::Confirmed,
    //             }
    //         })
    //     })
    //     .unwrap();
    //     let mut tx = Transaction::new_with_payer(&ixs, Some(&payer.pubkey()));
    //     tx.sign(&[payer], latest_blockhash);
    //     // Send to jito client
    //     // jito_client.send_transaction_with_config(
    //     //     &tx,
    //     //     RpcSendTransactionConfig {
    //     //         skip_preflight: true,
    //     //         max_retries: Some(0),
    //     //         ..RpcSendTransactionConfig::default()
    //     //     },
    //     // ).unwrap();
    //
    //     println!("Try number {}", try_number);
    //     try_number += 1;
    //     // Check if the blockhash has expired
    //     let is_valid;
    //     let is_valid_result = retry_rpc(|| {
    //         client.is_blockhash_valid(
    //             &latest_blockhash,
    //             CommitmentConfig {
    //                 commitment: CommitmentLevel::Confirmed,
    //             },
    //         )
    //     });
    //     match is_valid_result {
    //         Ok(is_valid_value) => {
    //             is_valid = is_valid_value;
    //         }
    //         Err(_) => continue,
    //     }
    //     println!("Is blockhash valid: {:?}", is_valid);
    //     let sent = client.send_transaction_with_config(
    //         &tx,
    //         RpcSendTransactionConfig {
    //             skip_preflight: true,
    //             max_retries: Some(0),
    //             ..RpcSendTransactionConfig::default()
    //         },
    //     );
    //     if let Some(sig) = sent.ok() {
    //         println!("Sent transaction: {:?}", sig);
    //         let result = client.confirm_transaction_with_spinner(
    //             &sig,
    //             &latest_blockhash,
    //             CommitmentConfig {
    //                 commitment: CommitmentLevel::Confirmed,
    //             },
    //         );
    //         match result {
    //             Ok(()) => {
    //                 println!("Confirmed. Delaying so next instruction will work");
    //                 std::thread::sleep(std::time::Duration::from_secs(10));
    //                 return Ok(sig);
    //             }
    //             Err(_e) => continue,
    //         }
    //     }
    //     println!("Trying again")
    // }
}
