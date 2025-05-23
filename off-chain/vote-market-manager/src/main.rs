use std::str::FromStr;
use std::{env, fs};

use anchor_lang::AnchorDeserialize;
use chrono::Utc;
use clap::value_parser;
use clap::ArgAction::SetTrue;
use dotenv::dotenv;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use crate::accounts::resolve::{get_delegate, get_escrow_address_for_owner};
use crate::actions::management::data::VoteInfo;
use crate::actions::queries::escrows;
use crate::utils::{create_logger, short_address};

mod accounts;
mod actions;
mod errors;
mod utils;

const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;
const GAUGEMEISTER: Pubkey = pubkey!("28ZDtf6d2wsYhBvabTxUHTRT6MDxqjmqR7RMCp348tyU");

const ADMIN: Pubkey = pubkey!("AmbWk325Nr67A5wpoHnxh967Zf4C5fQP9KHE3eeJQYWU");
const LOCKER: Pubkey = pubkey!("8erad8kmNrLJDJPe9UkmTHomrMV3EW48sjGeECyVjbYX");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    create_logger()?;
    let cmd = clap::Command::new("vote-market-manager")
        .bin_name("vote-market-manager")
        .arg(
            clap::Arg::new("keypair")
                .long("keypair")
                .short('k')
                .required(false)
                .value_parser(value_parser!(String))
                .help("The keypair to use for the payer")
                .global(true),
        )
        .subcommand(
            clap::command!("get-escrows").arg(
                clap::Arg::new("config")
                    .required(true)
                    .value_parser(value_parser!(String))
                    .help("The config to calculate the escrow delegate"),
            ),
        )
        .subcommand(
            clap::command!("get-escrow-votes")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config to calculate the escrow delegate"),
                )
                .arg(
                    clap::Arg::new("gauge")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The gauge to get the votes for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to get the votes for"),
                ),
        )
        .subcommand(
            clap::command!("get-direct-votes").arg(
                clap::Arg::new("epoch")
                    .required(true)
                    .value_parser(value_parser!(u32))
                    .help("The epoch to get the direct votes for"),
            ),
        )
        .subcommand(
            clap::command!("get-vote-buys")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("the config for the vote buy accounts"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("the epoch for the vote buy accounts"),
                ),
        )
        .subcommand(clap::command!("get-gauges"))
        .subcommand(
            clap::command!("get-escrow").arg(
                clap::Arg::new("owner")
                    .required(true)
                    .value_parser(value_parser!(String))
                    .help("The owner of the escrow to get"),
            ),
        )
        .subcommand(
            clap::command!("delegate").arg(
                clap::Arg::new("config")
                    .required(true)
                    .value_parser(value_parser!(String))
                    .help("The config to generate the delegate from"),
            ),
        )
        .subcommand(
            clap::command!("reset-epoch-gauge-voter")
                .arg(
                    clap::Arg::new("owner")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The owner of the escrow to reset the epoch gauge voter for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to reset the epoch gauge voter for"),
                ),
        )
        .subcommand(
            clap::command!("withdraw-votes")
                .arg(
                    clap::Arg::new("owner")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The owner of the escrow to withdraw votes for"),
                )
                .arg(
                    clap::Arg::new("gauge")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The gauge to withdraw votes for"),
                )
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config to withdraw votes for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to withdraw votes for"),
                ),
        )
        .subcommand(
            clap::command!("prepare-vote")
                .arg(
                    clap::Arg::new("owner")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The owner of the escrow to vote for"),
                )
                .arg(
                    clap::Arg::new("gauge")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The gauge voting for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to vote for"),
                ),
        )
        .subcommand(
            clap::command!("create-epoch-gauge")
                .arg(
                    clap::Arg::new("gauge")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The gauge to create the epoch gauge for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to create the gauge for"),
                ),
        )
        .subcommand(
            clap::command!("vote-test")
                .arg(
                    clap::Arg::new("owner")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The owner of the escrow to vote for"),
                )
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config for the vote buy accounts"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to vote for"),
                ),
        )
        .subcommand(
            clap::command!("check-votes").arg(
                clap::Arg::new("epoch")
                    .required(true)
                    .value_parser(value_parser!(u32))
                    .help("The epoch to check the votes for"),
            ),
        )
        .subcommand(
            clap::command!("clear-votes")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config to clear the votes for"),
                )
                .arg(
                    clap::Arg::new("owner")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The owner of the escrow to clear the votes for"),
                ),
        )
        .subcommand(
            clap::command!("setup").arg(
                clap::Arg::new("mints")
                    .long("mints")
                    .short('m')
                    .required(false)
                    .value_delimiter(',')
                    .value_parser(value_parser!(String))
                    .help("The mints to allow for the vote buys"),
            ),
        )
        .subcommand(
            clap::command!("update-mints")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config to update the mints for"),
                )
                .arg(
                    clap::Arg::new("mints")
                        .long("mints")
                        .short('m')
                        .required(true)
                        .value_delimiter(',')
                        .value_parser(value_parser!(String))
                        .help("The mints to allow for the vote buys"),
                ),
        )
        .subcommand(
            clap::command!("change-admin")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config to change the admin for"),
                )
                .arg(
                    clap::Arg::new("new_admin")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The new admin for the config"),
                ),
        )
        .subcommand(
            clap::command!("change-script-authority")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config to change the script authority for"),
                )
                .arg(
                    clap::Arg::new("new_script_authority")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The new script authority for the config"),
                ),
        )
        .subcommand(clap::command!("create-token"))
        .subcommand(
            clap::command!("buy-votes")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config for the vote buy accounts"),
                )
                .arg(
                    clap::Arg::new("gauge")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The gauge buy votes for"),
                )
                .arg(
                    clap::Arg::new("mint")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The amount of tokens to buy votes for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to vote for"),
                )
                .arg(
                    clap::Arg::new("amount")
                        .required(true)
                        .value_parser(value_parser!(u64))
                        .help("The amount of tokens to buy votes for"),
                ),
        )
        .subcommand(
            clap::command!("get-refund")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config for the vote buy accounts"),
                )
                .arg(
                    clap::Arg::new("gauge")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The gauge buy votes for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to vote for"),
                ),
        )
        .subcommand(
            clap::command!("set-maximum")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config for the vote buy accounts"),
                )
                .arg(
                    clap::Arg::new("gauge")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The gauge buy votes for"),
                )
                .arg(
                    clap::Arg::new("max")
                        .required(true)
                        .value_parser(value_parser!(u64))
                        .help("The maximum amount of tokens to buy votes for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to vote for"),
                ),
        )
        .subcommand(clap::command!("trigger-epoch"))
        .subcommand(
            clap::command!("claim")
                .arg(
                    clap::Arg::new("mint")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The mint for the token to claim"),
                )
                .arg(
                    clap::Arg::new("escrow")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The escrow to claim for"),
                )
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config for the vote buy accounts"),
                )
                .arg(
                    clap::Arg::new("gauge")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The gauge claim vote payments for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to vote for"),
                ),
        )
        .subcommand(
            clap::command!("calculate-inputs")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config for the vote buy accounts"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to calculate inputs for"),
                ),
        )
        .subcommand(
            clap::command!("calculate-weights").arg(
                clap::Arg::new("epoch-data")
                    .required(true)
                    .value_parser(value_parser!(String))
                    .help("The data file output by the calculate-inputs subcommand"),
            ),
        )
        .subcommand(
            clap::command!("find-max-vote-buy")
                .arg(
                    clap::Arg::new("epoch-data")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The data file output by the calculate-inputs subcommand"),
                )
                .arg(
                    clap::Arg::new("vote-weights")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The vote weights file output by the calculate-weights subcommand"),
                )
                .arg(
                    clap::Arg::new("dry-run")
                        .required(false)
                        .long("dry-run")
                        .short('d')
                        .action(SetTrue)
                        .help("Whether to actually set the maximums"),
                )
                .arg(
                    clap::Arg::new("use-all")
                        .required(false)
                        .long("use-all")
                        .short('u')
                        .action(SetTrue)
                        .help("Set to use all tokens even if valued higher than emissions"),
                ),
        )
        .subcommand(
            clap::command!("execute-votes")
                .arg(
                    clap::Arg::new("epoch-data")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The data file output by the calculate-inputs subcommand"),
                )
                .arg(
                    clap::Arg::new("vote-weights")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The vote weights file output by the calculate-weights subcommand"),
                )
                .arg(
                    clap::Arg::new("escrow")
                        .required(false)
                        .long("escrow")
                        .short('e')
                        .value_parser(value_parser!(String))
                        .help("Owner of a single escrow to execute votes for"),
                ),
        )
        .subcommand(
            clap::command!("create-parallel-sh")
                .arg(
                    clap::Arg::new("epoch-data")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The data file output by the calculate-inputs subcommand"),
                )
                .arg(
                    clap::Arg::new("vote-weights")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The vote weights file output by the calculate-weights subcommand"),
                ),
        )
        .subcommand(
            clap::command!("execute-claim")
                .arg(
                    clap::Arg::new("config")
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("The config to claim for"),
                )
                .arg(
                    clap::Arg::new("epoch")
                        .required(true)
                        .value_parser(value_parser!(u32))
                        .help("The epoch to claim for"),
                ),
        );

    let matches = cmd.get_matches();
    let keypair = matches.get_one::<String>("keypair");
    let keypair_path = match keypair {
        Some(keypair) => keypair.to_string(),
        None => env::var("KEY_PATH2")?.to_string(),
    };
    let rpc_url = env::var("RPC_URL").unwrap().to_string();
    let rpc_wss_url = env::var("RPC_WSS_URL").unwrap().to_string();
    println!("rpc_url: {:?}", rpc_url);
    let client = solana_client::rpc_client::RpcClient::new(&rpc_url);
    let payer = solana_sdk::signature::read_keypair_file(keypair_path)?;
    println!("payer: {:?}", payer.pubkey());
    let anchor_client = anchor_client::Client::new_with_options(
        anchor_client::Cluster::Custom(rpc_url.clone(), rpc_wss_url),
        &payer,
        CommitmentConfig::confirmed(),
    );
    let program = anchor_client.program(vote_market::id())?;
    if rpc_url == "http://127.0.0.1:8899" || rpc_url == "http://localhost:8899" {
        // Make sure we have some funds
        let amount = program.rpc().get_balance(&payer.pubkey())?;
        if amount == 0 {
            println!("Airdropping 100 SOL");
            let sig = program
                .rpc()
                .request_airdrop(&payer.pubkey(), 100_000_000_000)?;
            let blockhash = program.rpc().get_latest_blockhash()?;
            program.rpc().confirm_transaction_with_spinner(
                &sig,
                &blockhash,
                CommitmentConfig {
                    commitment: solana_sdk::commitment_config::CommitmentLevel::Finalized,
                },
            )?;
        }
    }
    match matches.subcommand() {
        Some(("get-escrows", matches)) => {
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let delegate = get_delegate(&config);
            let escrows = escrows::get_delegated_escrows(&client, &delegate);
            println!("escrows: {:?}", escrows);
            println!("Number of escrows: {:?}", escrows.len());
        }
        Some(("get-escrow-votes", matches)) => {
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let gauge = Pubkey::from_str(matches.get_one::<String>("gauge").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            let delegate = get_delegate(&config);
            println!("Epoch {:?}", epoch);
            escrows::get_escrow_votes(&client, &delegate, &gauge, *epoch);
            println!("done");
        }
        Some(("delegate", matches)) => {
            let escrow = get_escrow_address_for_owner(&payer.pubkey());
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let delegate = accounts::resolve::get_delegate(&config);
            println!("delegate: {:?}", delegate);
            actions::delegate::delegate(client, &escrow, &delegate, &payer);
        }
        Some(("reset-epoch-gauge-voter", matches)) => {
            let owner = Pubkey::from_str(matches.get_one::<String>("owner").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            actions::reset_epoch_gauge_voter::reset_epoch_gauge_voter(
                &client, &payer, owner, *epoch,
            );
        }
        Some(("withdraw-votes", matches)) => {
            let owner = Pubkey::from_str(matches.get_one::<String>("owner").unwrap())?;
            let gauge = Pubkey::from_str(matches.get_one::<String>("gauge").unwrap())?;
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            actions::withdraw_votes::withdraw_votes(&client, &payer, owner, gauge, config, *epoch);
        }
        Some(("get-escrow", matches)) => {
            let owner = Pubkey::from_str(matches.get_one::<String>("owner").unwrap())?;
            let escrow = accounts::resolve::get_escrow_address_for_owner(&owner);
            println!("{}", escrow);
        }
        Some(("get-direct-votes", matches)) => {
            // Update this with the latest from the file from calculate-inputs
            let sbr_price = 0.005778;
            let sbr_per_epoch = 7000000;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            let direct_votes = actions::queries::direct_votes::get_direct_votes(&client, *epoch)?;
            let total_votes = direct_votes.iter().map(|x| x.total_power).sum::<u64>();
            let usd_per_vote = sbr_price * sbr_per_epoch as f64 / total_votes as f64;
            println!("USD per vote: {:?}", usd_per_vote);
            for eg in direct_votes {
                let gauge_account = client.get_account(&eg.gauge)?;
                let gauge_data =
                    gauge_state::Gauge::deserialize(&mut gauge_account.data[8..].as_ref())?;
                let quarry_address = gauge_data.quarry;
                let quarry_account = client.get_account(&quarry_address)?;
                let quarry_data =
                    quarry_state::Quarry::deserialize(&mut quarry_account.data[8..].as_ref())?;
                let quarry_mint = quarry_data.token_mint_key;
                println!(
                    "Pool Mint: {:?}, Gauge: {:?}, Power: {:?}, USD value of votes: {:?}",
                    short_address(&quarry_mint),
                    &eg.gauge,
                    eg.total_power,
                    eg.total_power as f64 * usd_per_vote
                );
            }
            println!("Total votes: {:?}", total_votes);
        }
        Some(("get-vote-buys", matches)) => {
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            let vote_buys = actions::queries::vote_buys::get_all_vote_buys(*epoch, &config);
            println!("vote buys: {:?}", vote_buys);
        }
        Some(("get-gauges", _)) => {
            actions::queries::get_gauges::get_gauges(client);
        }
        Some(("prepare-vote", matches)) => {
            println!("prepare-vote");
            let owner = Pubkey::from_str(matches.get_one::<String>("owner").unwrap())?;
            let gauge = Pubkey::from_str(matches.get_one::<String>("gauge").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            actions::prepare_vote::prepare_vote(&client, owner, gauge, &payer, *epoch);
        }
        Some(("create-epoch-gauge", matches)) => {
            println!("create-epoch-gauge");
            let gauge = Pubkey::from_str(matches.get_one::<String>("gauge").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            actions::create_epoch_gauge::create_epoch_gauge(&client, &payer, gauge, *epoch);
        }
        Some(("vote-test", matches)) => {
            println!("vote-test");
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let owner = Pubkey::from_str(matches.get_one::<String>("owner").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            let weights = vec![VoteInfo {
                gauge: Pubkey::from_str("3xC4eW6xhW3Gpb4T5sCKFe73ay2K4aUUfxL57XFdguJx")?,
                weight: 100,
                votes: 100,
            }];
            actions::vote_market::vote::vote(
                &anchor_client,
                &client,
                &payer,
                config,
                owner,
                *epoch,
                weights,
                false,
            )?;
        }
        Some(("check-votes", matches)) => {
            println!("check-votes");
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            actions::management::check_votes::check_votes(&client, &payer, *epoch)?;
        }
        Some(("clear-votes", matches)) => {
            println!("clear-votes");
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let owner = Pubkey::from_str(matches.get_one::<String>("owner").unwrap())?;
            actions::vote_market::clear_votes::clear_votes(
                &anchor_client,
                &client,
                &payer,
                config,
                owner,
            )?;
        }
        Some(("setup", matches)) => {
            println!("setup");
            let mut mints = vec![Pubkey::default()];
            if let Some(mint_vaulues) = matches.get_many::<String>("mints") {
                mints = mint_vaulues
                    .map(|mint| Pubkey::from_str(mint).unwrap())
                    .collect();
            }
            println!("got here");
            actions::vote_market::setup::setup(&anchor_client, mints, &payer);
        }
        Some(("update-mints", matches)) => {
            println!("update-mints-ok");
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            println!("config {:?}", config);
            let mut mints = vec![Pubkey::default()];
            println!("{:?}", mints);
            if let Some(mint_vaulues) = matches.get_many::<String>("mints") {
                mints = mint_vaulues
                    .map(|mint| Pubkey::from_str(mint).unwrap())
                    .collect();
            }
            println!("mints: {:?}", mints);
            actions::vote_market::update_mints::update_mints(
                &anchor_client,
                &client,
                &payer,
                config,
                mints,
            );
        }
        Some(("create-token", _)) => {
            println!("create-token");
            actions::create_token::create_token(&client, &payer);
        }
        Some(("buy-votes", matches)) => {
            //TODO: bring out epoch
            println!("buy-votes");
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let gauge = Pubkey::from_str(matches.get_one::<String>("gauge").unwrap())?;
            let mint = Pubkey::from_str(matches.get_one::<String>("mint").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            let amount = matches.get_one::<u64>("amount").unwrap();
            actions::vote_market::buy_votes::buy_votes(
                &anchor_client,
                &client,
                &payer,
                &config,
                &gauge,
                &mint,
                *epoch,
                *amount,
            );
        }
        Some(("get-refund", matches)) => {
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let gauge = Pubkey::from_str(matches.get_one::<String>("gauge").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            actions::vote_market::refund::get_refund(
                &anchor_client,
                &client,
                &payer,
                config,
                gauge,
                *epoch,
            );
        }
        Some(("set-maximum", matches)) => {
            //TODO: bring out epoch
            let maximum = matches.get_one::<u64>("max").unwrap();
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let gauge = Pubkey::from_str(matches.get_one::<String>("gauge").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            actions::vote_market::set_maximum::set_maximum(
                &anchor_client,
                &client,
                &payer,
                gauge,
                config,
                *epoch,
                *maximum,
            );
        }
        Some(("trigger-epoch", _)) => {
            actions::trigger_epoch::trigger_epoch(&client, &payer);
        }
        Some(("claim", matches)) => {
            let mint = Pubkey::from_str(matches.get_one::<String>("mint").unwrap())?;
            let escrow = Pubkey::from_str(matches.get_one::<String>("escrow").unwrap())?;
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let gauge = Pubkey::from_str(matches.get_one::<String>("gauge").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();

            actions::vote_market::claim::claim(
                &anchor_client,
                &client,
                &payer,
                payer.pubkey(),
                mint,
                escrow,
                config,
                gauge,
                *epoch,
            )?;
        }
        Some(("calculate-inputs", matches)) => {
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let mut db_client = utils::connect_to_db()?;
            let filename = actions::management::calculate_inputs::calculate_inputs(
                &client,
                &mut db_client,
                &config,
                *epoch,
            )?;
            println!("{}", filename);
        }
        Some(("calculate-weights", matches)) => {
            let epoch_data = matches.get_one::<String>("epoch-data").unwrap();
            let epoch_data_string = fs::read_to_string(epoch_data)?;
            let mut data: actions::management::data::EpochData =
                serde_json::from_str(&epoch_data_string)?;
            let results = actions::management::calculate_weights::calculate_weights(&mut data)?;
            println!("results {:?}", results);
            let vote_weights_json = serde_json::to_string(&results).unwrap();
            let filename = format!(
                "./epoch_{}_weights{}.json",
                data.epoch,
                Utc::now().format("%Y-%m-%d-%H_%M")
            );
            fs::write(&filename, vote_weights_json)?;
            println!("{}", filename);
        }
        Some(("find-max-vote-buy", matches)) => {
            println!("Finding and setting max vote buy");
            let epoch_data = matches.get_one::<String>("epoch-data").unwrap();
            let epoch_data_string = std::fs::read_to_string(epoch_data)?;
            let data: actions::management::data::EpochData =
                serde_json::from_str(&epoch_data_string)?;
            let vote_weights_file = matches.get_one::<String>("vote-weights").unwrap();
            let vote_weights_string = std::fs::read_to_string(vote_weights_file)?;
            let vote_weights: Vec<VoteInfo> = serde_json::from_str(&vote_weights_string)?;
            println!("got here");
            let dry_run = *matches.get_one::<bool>("dry-run").unwrap();
            let use_all = *matches.get_one::<bool>("use-all").unwrap();
            println!("dry-run: {:?}", dry_run);
            actions::management::find_max_vote_buy::find_max_vote_buy(
                &client,
                &anchor_client,
                &payer,
                data,
                vote_weights,
                dry_run,
                use_all,
            )?;
        }
        Some(("execute-votes", matches)) => {
            println!("Executing votes");
            let epoch_data = matches.get_one::<String>("epoch-data").unwrap();
            let epoch_data_string = std::fs::read_to_string(epoch_data)?;
            let data: actions::management::data::EpochData =
                serde_json::from_str(&epoch_data_string)?;
            let vote_weights_file = matches.get_one::<String>("vote-weights").unwrap();
            let vote_weights_string = std::fs::read_to_string(vote_weights_file)?;
            let vote_infos: Vec<VoteInfo> = serde_json::from_str(&vote_weights_string)?;
            let escrow = match matches.get_one::<String>("escrow") {
                Some(escrow) => Some(Pubkey::from_str(escrow)?),
                None => None,
            };
            actions::management::execute_votes::execute_votes(
                &client,
                &anchor_client,
                &payer,
                data,
                vote_infos,
                escrow,
            )?;
        }
        Some(("create-parallel-sh", matches)) => {
            println!("Creating parallel script");
            let epoch_data = matches.get_one::<String>("epoch-data").unwrap();
            let epoch_data_string = fs::read_to_string(epoch_data)?;
            let data: actions::management::data::EpochData =
                serde_json::from_str(&epoch_data_string)?;
            let vote_weights_file = matches.get_one::<String>("vote-weights").unwrap();
            actions::management::create_parallel_sh::create_parallel_sh(
                epoch_data,
                vote_weights_file,
                data,
            )?;
        }
        Some(("execute-claim", matches)) => {
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let epoch = matches.get_one::<u32>("epoch").unwrap();
            actions::management::execute_claim::execute_claim(
                &client,
                &anchor_client,
                &payer,
                config,
                *epoch,
            )?;
        }
        Some(("change-admin", matches)) => {
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let new_admin = Pubkey::from_str(matches.get_one::<String>("new_admin").unwrap())?;
            actions::vote_market::change_admin::change_admin(
                &client,
                &anchor_client,
                &payer,
                config,
                new_admin,
            )?;
        }
        Some(("change-script-authority", matches)) => {
            let config = Pubkey::from_str(matches.get_one::<String>("config").unwrap())?;
            let new_script_authority =
                Pubkey::from_str(matches.get_one::<String>("new_script_authority").unwrap())?;
            actions::vote_market::change_script_authority::change_script_authority(
                &client,
                &anchor_client,
                &payer,
                config,
                new_script_authority,
            )?;
        }
        _ => {
            println!("No subcommand");
        }
    };
    Ok(())
}
