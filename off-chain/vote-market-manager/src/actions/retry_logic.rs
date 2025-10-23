use std::env;
use crate::actions::goki::OwnerInvokeInstructionV2;
use crate::actions::lookup_table::get_lookup_tables;
use crate::actions::rpc_retry::retry_rpc;
use crate::errors::VoteMarketManagerError::SimulationFailed;
use anchor_lang::InstructionData;
use openssl::base64;
use reqwest::blocking::Client;
use retry::delay::Fixed;
use retry::Error as RetryError;
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::message::{v0, VersionedMessage};
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::{Transaction, VersionedTransaction};
use solana_sdk::{pubkey};
use tokio::time::{Duration};

pub fn retry_logic_goki<'a>(
    client: &'a RpcClient,
    payer: &'a Keypair,
    ixs: &'a mut Vec<Instruction>,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let staked_rpc = env::var("STAKED_RPC").unwrap().to_string();
    let send_client = RpcClient::new(staked_rpc);

    let invoker = pubkey!("AMd2nnFYtPGkeEbUvyVtWRDkG3nrESCvNW4C43mEvWrF");
    let goki_program_id = pubkey!("GokivDYuQXPZCWRkwMhdH2h91KpDQXBEmpgBgs55bnpH");
    let smart_wallet = pubkey!("Eh7BJiZVxJ5bv9XA7NGS5UQTHmt1eZGb6aVFdSCT8XMg");
    let goki_ixs: Vec<Instruction> = ixs
        .into_iter()
        .map(|ix| {
            let updated_accounts: Vec<AccountMeta> = ix
                .accounts
                .iter()
                .map(|acc| {
                    if acc.pubkey == payer.pubkey() {
                        AccountMeta {
                            pubkey: invoker,
                            is_signer: false,
                            is_writable: acc.is_writable,
                        }
                    } else {
                        acc.clone()
                    }
                })
                .collect();
            let mut accounts = vec![
                AccountMeta {
                    pubkey: smart_wallet,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: payer.pubkey(),
                    is_signer: true,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: ix.program_id,
                    is_signer: false,
                    is_writable: false,
                },
            ];
            accounts.extend_from_slice(&updated_accounts);
            let data = OwnerInvokeInstructionV2 {
                index: 0,
                bump: 254,
                invoker,
                data: ix.data.clone(),
            };
            Instruction {
                program_id: goki_program_id,
                accounts,
                data: data.data(),
            }
        })
        .collect();

    let mut sim_tx = Transaction::new_with_payer(&goki_ixs, Some(&payer.pubkey()));
    let sim_strategy = Fixed::from_millis(1000).take(5);
    println!("going to sim");
    let sim_result = retry::retry(sim_strategy, || {
        let latest_blockhash = retry_rpc(|| {
            client.get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
        });
        println!("got blockhash {:?}", latest_blockhash);
        println!("really going to sim");
        let sim = retry_rpc(|| {
            client.simulate_transaction_with_config(&sim_tx, {
                RpcSimulateTransactionConfig {
                    replace_recent_blockhash: true,
                    sig_verify: false,
                    commitment: Some(CommitmentConfig::confirmed()),
                    ..RpcSimulateTransactionConfig::default()
                }
            })
        });
        println!("got sim");
        match sim {
            Ok(sim) => {
                println!("simulated: {:?}", sim);
                if sim.value.err.is_some() {
                    println!("Internal error simulating transaction, retry");
                    return Err(RetryError {
                        tries: 0,
                        total_delay: std::time::Duration::from_millis(0),
                        error: "RPC failed to simulate transaction".to_string(),
                    });
                }
                Ok(sim)
            }
            Err(e) => {
                println!("Error result simulating transaction, retry {:?}", e);
                Err(RetryError {
                    tries: 0,
                    total_delay: std::time::Duration::from_millis(0),
                    error: "RPC failed to simulate transaction".to_string(),
                })
            }
        }
    });
    let cus = match sim_result {
        Ok(sim) => {
            println!("simulated: {:?}", sim);
            if sim.value.err.is_some() {
                println!("SIM ERROR: {:?}", sim.value.err);
                return Err(Box::new(SimulationFailed {
                    sim_info: sim.value.logs.unwrap().join(" "),
                }));
            } else if let Some(cus) = sim.value.units_consumed {
                cus
            } else {
                return Err(Box::new(SimulationFailed {
                    sim_info: "No units consumed".to_string(),
                }));
            }
        }
        Err(e) => {
            //TODO: MAke a proper error
            println!("Error simulating transaction: {:?}", e);
            return Err(Box::new(SimulationFailed {
                sim_info: "Error simulating transaction".to_string(),
            }));
        }
    };
    let mut tries = 0;
    println!("Consumed units: {}", cus);

    loop {
        let http_client = Client::new();
        let blockhash = client.get_latest_blockhash()?;
        let fee_tx = VersionedTransaction::try_new(
            VersionedMessage::V0(v0::Message::try_compile(
                &payer.pubkey(),
                &goki_ixs.clone(),
                &get_lookup_tables(),
                blockhash,
            )?),
            &[payer],
        )?;
        let b64_tx = base64::encode_block(&bincode::serialize(&fee_tx).unwrap());
        let body = json!({
            "jsonrpc": "2.0",
            "id": "vota",
            "method": "getPriorityFeeEstimate",
            "params": [
                {
                    "transaction": &b64_tx,
                    "options": {
                        "transactionEncoding": "base64",
                        "recommended": true,
                    },
                }
            ]
        });
        println!("Trying to get prio fee");
        let response = http_client
            .post("https://vota.boundlessendeavors.dev") // Replace with your RPC endpoint
            .json(&body)
            .send()
            .unwrap()
            .text();
        let data: Value = serde_json::from_str(&response.unwrap().to_string()).unwrap();
        println!("Response: {:?}", &data);

        let priority_fee : Value = data["result"]["priorityFeeEstimate"].clone();
        let f64_priority_fee = priority_fee.as_f64().unwrap();
        let ulamports_per_cu = ((f64_priority_fee * 1000000.0) as u64) / (cus + 2000);
        println!("Ulamports per CU: {:?}", ulamports_per_cu);
        let cus_u32: u32 = cus.try_into().map_err(|_| SimulationFailed {
            sim_info: "Failed to convert cus from u64 to u32".to_string(),
        })?;
        let mut ixs: Vec<Instruction> = vec![
            ComputeBudgetInstruction::set_compute_unit_limit(cus_u32 + 2000),
            ComputeBudgetInstruction::set_compute_unit_price(ulamports_per_cu),
        ];
        ixs.append(&mut goki_ixs.clone());

        let tx = VersionedTransaction::try_new(
            VersionedMessage::V0(v0::Message::try_compile(
                &payer.pubkey(),
                &ixs.clone(),
                &get_lookup_tables(),
                blockhash,
            )?),
            &[payer],
        )?;
        println!("Going to send");
        let result = send_client.send_transaction_with_config(
            &tx,
            RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentLevel::Confirmed),
                encoding: None,
                max_retries: Some(0),
                min_context_slot: None,
            },
        );
        println!("Transaction sent: {:?}", result);
        match result {
            Ok(sig) => {
                let confirm_result = client.confirm_transaction_with_spinner(
                    &sig,
                    &blockhash,
                    CommitmentConfig::confirmed(),
                );
                return match confirm_result {
                    Ok(_) => {
                        println!("Transaction confirmed: {:?}", sig);
                        Ok(sig)
                    }
                    Err(e) => {
                        println!("Error confirming transaction: {:?}", e);
                        Err(Box::new(e))
                    }
                };
            }
            Err(e) => {
                if tries == 10 {
                    println!("Error sending transaction: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        };
        tries += 1;
        println!("Retrying transaction {}", tries);
    }
}

pub fn retry_logic_direct<'a>(
    client: &'a RpcClient,
    payer: &'a Keypair,
    ixs: &'a Vec<Instruction>,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let staked_rpc = env::var("STAKED_RPC").unwrap().to_string();
    let send_client = RpcClient::new(staked_rpc);

    // Simulate transaction first
    let mut sim_tx = Transaction::new_with_payer(ixs, Some(&payer.pubkey()));
    let sim_strategy = Fixed::from_millis(1000).take(5);
    let sim_result = retry::retry(sim_strategy, || {
        let sim = retry_rpc(|| {
            client.simulate_transaction_with_config(&sim_tx, {
                RpcSimulateTransactionConfig {
                    replace_recent_blockhash: true,
                    sig_verify: false,
                    commitment: Some(CommitmentConfig::confirmed()),
                    ..RpcSimulateTransactionConfig::default()
                }
            })
        });
        match sim {
            Ok(sim) => {
                println!("simulated: {:?}", sim);
                if sim.value.err.is_some() {
                    println!("Internal error simulating transaction, retry");
                    return Err(RetryError {
                        tries: 0,
                        total_delay: std::time::Duration::from_millis(0),
                        error: "RPC failed to simulate transaction".to_string(),
                    });
                }
                Ok(sim)
            }
            Err(e) => {
                println!("Error result simulating transaction, retry {:?}", e);
                Err(RetryError {
                    tries: 0,
                    total_delay: std::time::Duration::from_millis(0),
                    error: "RPC failed to simulate transaction".to_string(),
                })
            }
        }
    });

    let cus = match sim_result {
        Ok(sim) => {
            println!("simulated: {:?}", sim);
            if sim.value.err.is_some() {
                println!("SIM ERROR: {:?}", sim.value.err);
                return Err(Box::new(SimulationFailed {
                    sim_info: sim.value.logs.unwrap().join(" "),
                }));
            } else if let Some(cus) = sim.value.units_consumed {
                cus
            } else {
                return Err(Box::new(SimulationFailed {
                    sim_info: "No units consumed".to_string(),
                }));
            }
        }
        Err(e) => {
            println!("Error simulating transaction: {:?}", e);
            return Err(Box::new(SimulationFailed {
                sim_info: "Error simulating transaction".to_string(),
            }));
        }
    };

    let mut tries = 0;
    println!("Consumed units: {}", cus);

    loop {
        let http_client = Client::new();
        let blockhash = client.get_latest_blockhash()?;
        
        // Create fee estimation transaction
        let fee_tx = VersionedTransaction::try_new(
            VersionedMessage::V0(v0::Message::try_compile(
                &payer.pubkey(),
                ixs,
                &get_lookup_tables(),
                blockhash,
            )?),
            &[payer],
        )?;
        let b64_tx = base64::encode_block(&bincode::serialize(&fee_tx).unwrap());
        let body = json!({
            "jsonrpc": "2.0",
            "id": "vota",
            "method": "getPriorityFeeEstimate",
            "params": [
                {
                    "transaction": &b64_tx,
                    "options": {
                        "transactionEncoding": "base64",
                        "recommended": true,
                    },
                }
            ]
        });
        
        println!("Trying to get prio fee");
        let response = http_client
            .post("https://vota.boundlessendeavors.dev")
            .json(&body)
            .send()
            .unwrap()
            .text();
        let data: Value = serde_json::from_str(&response.unwrap().to_string()).unwrap();
        println!("Response: {:?}", &data);

        let priority_fee: Value = data["result"]["priorityFeeEstimate"].clone();
        let f64_priority_fee = priority_fee.as_f64().unwrap();
        let ulamports_per_cu = ((f64_priority_fee * 1000000.0) as u64) / (cus + 2000);
        println!("Ulamports per CU: {:?}", ulamports_per_cu);
        
        let cus_u32: u32 = cus.try_into().map_err(|_| SimulationFailed {
            sim_info: "Failed to convert cus from u64 to u32".to_string(),
        })?;
        
        // Build final transaction with compute budget instructions
        let mut final_ixs: Vec<Instruction> = vec![
            ComputeBudgetInstruction::set_compute_unit_limit(cus_u32 + 2000),
            ComputeBudgetInstruction::set_compute_unit_price(ulamports_per_cu),
        ];
        final_ixs.extend_from_slice(ixs);

        let tx = VersionedTransaction::try_new(
            VersionedMessage::V0(v0::Message::try_compile(
                &payer.pubkey(),
                &final_ixs,
                &get_lookup_tables(),
                blockhash,
            )?),
            &[payer],
        )?;
        
        let result = send_client.send_transaction_with_config(
            &tx,
            RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentLevel::Confirmed),
                encoding: None,
                max_retries: Some(0),
                min_context_slot: None,
            },
        );
        
        println!("Transaction sent: {:?}", result);
        match result {
            Ok(sig) => {
                let confirm_result = client.confirm_transaction_with_spinner(
                    &sig,
                    &blockhash,
                    CommitmentConfig::confirmed(),
                );
                return match confirm_result {
                    Ok(_) => {
                        println!("Transaction confirmed: {:?}", sig);
                        Ok(sig)
                    }
                    Err(e) => {
                        println!("Error confirming transaction: {:?}", e);
                        Err(Box::new(e))
                    }
                };
            }
            Err(e) => {
                if tries == 10 {
                    println!("Error sending transaction: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        };
        tries += 1;
        println!("Retrying transaction {}", tries);
    }
}
