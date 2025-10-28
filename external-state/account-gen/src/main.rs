use crate::account::process_account;
use crate::toml_update::{update_anchor_toml, AddressInfo};
use anchor_lang::prelude::*;
use dotenv::dotenv;
use gauge_state::{
    EpochGauge, EpochGaugeVote, EpochGaugeVoter, Gauge, GaugeVote, GaugeVoter, Gaugemeister,
};
use locked_voter_state::{Escrow, Locker};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::signer::keypair::read_keypair_file;
use std::{env, fs};
use std::fmt::format;
use solana_sdk::pubkey;
use toml::{Table, Value};
use vote_market::state::VoteBuy;

mod account;
mod errors;
mod toml_update;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let cwd = env::current_dir().unwrap();
    let mut accounts_to_update: Vec<AddressInfo> = Vec::<AddressInfo>::new();
    // Make sure this is run from the project workspace directory
    if let Some(dirname) = cwd.iter().last().and_then(|osstr| osstr.to_str()) {
        if dirname == "account-gen" {
            println!("Must run from project root directory");
            return Err(Box::new(errors::AccountGenError::InvalidCwd));
        }
    }
    // create test accounts directory if it doesn't exist
    let test_accounts_dir = cwd.join("test-accounts");
    if !test_accounts_dir.exists() {
        fs::create_dir(test_accounts_dir)?;
    }
    let keypaths = vec!["KEY_PATH", "KEY_PATH2", "KEY_PATH3", "KEY_PATH4"];
    for set  in 0u32..2 {
        println!("Setting to {}", set);
        let payer = create_payer(keypaths[(set * 2) as usize])?;
        let payer2 = create_payer(keypaths[(set * 2 + 1) as usize])?;
        println!("Using payer pubkey: {:?}", payer.pubkey());
        println!("Using payer2 pubkey: {:?}", payer2.pubkey());

        let config_file = fs::read_to_string(format!("./tests/reward_config{}.json", set)).unwrap();
        let config_bytes: Vec<u8> = serde_json::from_str(&config_file).unwrap();
        let config = Keypair::from_bytes(config_bytes.as_slice()).unwrap();
        let (vote_delegate_address, _) = Pubkey::find_program_address(
            &[b"vote-delegate", config.pubkey().to_bytes().as_ref()],
            &vote_market::id(),
        );
        println!("Vote delegate address is {}", vote_delegate_address);
        let (gaugemeister_data, gaugemeister_account) = process_account::<Gaugemeister, _>(
            set,
            "gaugemeister",
            None,
            |mut data| {
                data.epoch_duration_seconds = 1;
                data
            },
            &mut accounts_to_update,
            "",
        )?;

        println!("next epoch is {}", gaugemeister_data.next_epoch_starts_at);

        let (_, gauge_account) =
            account::process_account::<Gauge, _>(set, "gauge", None, |x| x, &mut accounts_to_update, "")?;

        let (epoch_gauge_address, _) = Pubkey::find_program_address(
            &[
                b"EpochGauge",
                gauge_account.pubkey.to_bytes().as_ref(),
                (gaugemeister_data.voting_epoch()? + set * 3).to_le_bytes().as_ref(),
            ],
            &gauge_state::id(),
        );

        process_account::<EpochGauge, _>(
            set,
            "epoch-gauge",
            Some(epoch_gauge_address),
            |x| x,
            &mut accounts_to_update,
            "",
        )?;

        let config_file = fs::read_to_string(format!("./tests/reward_config{}.json", set))
            .expect("test config key should be provided in the repo");
        let config_bytes: Vec<u8> = serde_json::from_str(&config_file).unwrap();
        let config = Keypair::from_bytes(config_bytes.as_slice()).unwrap();
        // Create voting user
        let gauge_vote_address = create_user_votes(
            set,
            &mut accounts_to_update,
            &payer,
            &gaugemeister_data,
            &gaugemeister_account.pubkey,
            &gauge_account.pubkey,
            &config.pubkey(),
            "",
        )?;

        let config2_file = fs::read_to_string(format!("./tests/config{}.json", set + 2)).unwrap();
        let config2_bytes: Vec<u8> = serde_json::from_str(&config2_file).unwrap();
        let config2 = Keypair::from_bytes(config2_bytes.as_slice()).unwrap();
        // Create user that hasn't voted yet
        create_user_votes(
            set,
            &mut accounts_to_update,
            &payer2,
            &gaugemeister_data,
            &gaugemeister_account.pubkey,
            &gauge_account.pubkey,
            &config2.pubkey(),
            "-no-vote",
        )?;

        process_account::<Locker, _>(set, "locker", None, |data| data, &mut accounts_to_update, "")?;

        let (escrow_address, _) = Pubkey::find_program_address(
            &[
                b"Escrow",
                gaugemeister_data.locker.to_bytes().as_ref(),
                payer.pubkey().to_bytes().as_ref(),
            ],
            &locked_voter_state::id(),
        );
        println!("Escrow address is {}", escrow_address);
        let (gauge_voter_address, _) = Pubkey::find_program_address(
            &[
                b"GaugeVoter",
                gaugemeister_account.pubkey.to_bytes().as_ref(),
                escrow_address.to_bytes().as_ref(),
            ],
            &gauge_state::id(),
        );
        let (epoch_gauge_voter_address, _) = Pubkey::find_program_address(
            &[
                b"EpochGaugeVoter",
                gauge_voter_address.to_bytes().as_ref(),
                (gaugemeister_data.voting_epoch()? + set * 3).to_le_bytes().as_ref(),
            ],
            &gauge_state::id(),
        );
        println!("Epoch is {}", gaugemeister_data.voting_epoch()? + set * 2);
        println!("gauge voter address is {}", gauge_voter_address);
        let (epoch_gauge_voter_data, _) = process_account::<EpochGaugeVoter, _>(
            set,
            "epoch-gauge-voter",
            Some(epoch_gauge_voter_address),
            |mut data| {
                data.gauge_voter = gauge_voter_address;
                data.voting_epoch = gaugemeister_data
                    .voting_epoch()
                    .expect("if it deserializes the epoch should be valid") + set * 3;
                data
            },
            &mut accounts_to_update,
            "",
        )?;

        let (epoch_gauge_vote_address, _) = Pubkey::find_program_address(
            &[
                b"EpochGaugeVote",
                gauge_vote_address.to_bytes().as_ref(),
                epoch_gauge_voter_data.voting_epoch.to_le_bytes().as_ref(),
            ],
            &gauge_state::id(),
        );
        process_account::<EpochGaugeVote, _>(
            set,
            "epoch-gauge-vote",
            Some(epoch_gauge_vote_address),
            |x| x,
            &mut accounts_to_update,
            "",
        )?;


    }
    let anchor_toml_file = fs::read_to_string("./Anchor.toml").unwrap();
    let mut anchor_toml = Value::Table(anchor_toml_file.parse::<Table>().unwrap());
    update_anchor_toml(&mut anchor_toml, accounts_to_update);
    fs::copy("./Anchor.toml", "./Anchor.toml.bak")?;
    fs::write("./Anchor.toml", toml::to_string(&anchor_toml).unwrap())?;
    Ok(())
}

fn create_payer(key_path: &str) -> std::result::Result<Keypair, Box<dyn std::error::Error>> {
    let path = match env::var(key_path) {
        Ok(path) => path,
        Err(e) => {
            println!("No {} env variable set", key_path);
            return Err(Box::new(e));
        }
    };
    let payer = read_keypair_file(path)?;
    Ok(payer)
}

fn create_user_votes(
    set: u32,
    mut accounts_to_update: &mut Vec<AddressInfo>,
    payer: &Keypair,
    gaugemeister_data: &Gaugemeister,
    gaugemeister: &Pubkey,
    gauge: &Pubkey,
    config: &Pubkey,
    file_suffix: &str,
) -> std::result::Result<Pubkey, Box<dyn std::error::Error>> {
    let (escrow_address, _) = Pubkey::find_program_address(
        &[
            b"Escrow",
            gaugemeister_data.locker.to_bytes().as_ref(),
            payer.pubkey().to_bytes().as_ref(),
        ],
        &locked_voter_state::id(),
    );
    let (vote_delegate_address, _) = Pubkey::find_program_address(
        &[b"vote-delegate", config.to_bytes().as_ref()],
        &vote_market::id(),
    );

    println!("Vote delegate address is {}", vote_delegate_address);
    process_account::<Escrow, _>(
        set,
        "escrow",
        Some(escrow_address),
        |mut escrow_data| {
            escrow_data.owner = payer.pubkey();
            escrow_data.vote_delegate = vote_delegate_address;
            escrow_data.escrow_ends_at = 2_000_000_000;
            escrow_data
        },
        &mut accounts_to_update,
        file_suffix,
    )?;

    let (gauge_voter_address, _) = Pubkey::find_program_address(
        &[
            b"GaugeVoter",
            gaugemeister.to_bytes().as_ref(),
            escrow_address.to_bytes().as_ref(),
        ],
        &gauge_state::id(),
    );
    account::process_account::<GaugeVoter, _>(
        set,
        "gauge-voter",
        Some(gauge_voter_address),
        |mut data| {
            data.owner = payer.pubkey();
            data.escrow = escrow_address;
            data
        },
        &mut accounts_to_update,
        file_suffix,
    )?;

    let (gauge_vote_address, _) = Pubkey::find_program_address(
        &[
            b"GaugeVote",
            gauge_voter_address.to_bytes().as_ref(),
            gauge.to_bytes().as_ref(),
        ],
        &gauge_state::id(),
    );

    println!("Gauge vote address is {}", gauge_vote_address);
    process_account::<GaugeVote, _>(
        set,
        "gauge-vote",
        Some(gauge_vote_address),
        |mut data| {
            data.gauge_voter = gauge_voter_address;
            data.gauge = *gauge;
            data
        },
        &mut accounts_to_update,
        file_suffix,
    )?;


    let (vote_buy_address, _) = Pubkey::find_program_address(
        &[
            b"vote-buy",
            (gaugemeister_data.voting_epoch()? + set * 3).to_le_bytes().as_ref(),
            config.key().as_ref(),
            gauge.key().as_ref(),
        ],
        &vote_market::id(),
    );
    if file_suffix.eq("") {
        println!("Vote buy address is {}", vote_buy_address );
        process_account::<VoteBuy, _>(
            set,
            "vote-buy",
            Some(vote_buy_address),
            |mut data| {
                data.buyer = payer.pubkey();
                data.max_amount = Some(u64::MAX);
                data.gauge = *gauge;
                data.total_committed = 82_463_014_731;
                data.mint = pubkey!("GHDAAvZHR6rPyMMGD869ujyQ2UTTiuk9vMkq97xKqvNr"); // mint.json in the test directory
                data
            },
            &mut accounts_to_update,
            file_suffix,
        )?;
    }

    Ok(gauge_vote_address)
}
