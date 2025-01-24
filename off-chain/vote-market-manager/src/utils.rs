use crate::errors::VoteMarketManagerError;
use chrono::Utc;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres::Client;
use postgres_openssl::MakeTlsConnector;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use std::error::Error;
use std::path::Path;
use std::{env, fs};
use structured_logger::json::new_writer;
use structured_logger::Builder;

pub fn short_address(address: &Pubkey) -> String {
    let mut short = String::new();
    let address = address.to_string();
    let len = address.len();
    short.push_str(&address[..4]);
    short.push_str("...");
    short.push_str(&address[len - 4..]);
    short
}

pub fn get_multiple_accounts(client: &RpcClient, keys: Vec<Pubkey>) -> Vec<Option<Account>> {
    // get 50 accounts at a time
    let mut accounts: Vec<Option<Account>> = Vec::new();
    for keys_chunk in keys.chunks(50) {
        let accounts_chunk = client.get_multiple_accounts(keys_chunk).unwrap();
        accounts.extend(accounts_chunk);
    }
    accounts
}

pub fn create_logger() -> Result<(), Box<dyn Error>> {
    //Add a pid so parallel processes won't grab the same log file
    let pid = std::process::id();
    let log_dir = "./logs";
    if !Path::new(log_dir).exists() {
        fs::create_dir_all(log_dir)?;
    }
    Builder::with_level("info")
        .with_target_writer(
            "*",
            new_writer(fs::File::create(format!(
                "{}/vote_market_{}_{}.log",
                log_dir,
                Utc::now().format("%Y-%m-%d-%H_%M"),
                pid
            ))?),
        )
        .init();
    Ok(())
}

pub fn connect_to_db() -> Result<Client, Box<dyn Error>> {
    // Create Ssl postgres connector without verification
    let mut builder = SslConnector::builder(SslMethod::tls()).map_err(|e| {
        Box::new(VoteMarketManagerError::DatabaseConnection {
            error: e.to_string(),
        }) as Box<dyn Error>
    })?;
    builder.set_verify(SslVerifyMode::NONE);
    let connector = MakeTlsConnector::new(builder.build());

    // Connect to the PostgreSQL database
    let connection = Client::connect(
        &env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        connector,
    );
    match connection {
        Ok(client) => Ok(client),
        Err(e) => Err(Box::new(VoteMarketManagerError::DatabaseConnection {
            error: e.to_string(),
        })),
    }
}
