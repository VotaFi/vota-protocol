use crate::accounts::resolve::resolve_vote_keys;
use crate::utils::get_multiple_accounts;
use crate::{ANCHOR_DISCRIMINATOR_SIZE, GAUGEMEISTER, LOCKER};
use anchor_lang::AccountDeserialize;
use borsh::BorshDeserialize;
use gauge_state::{EpochGaugeVote, Gaugemeister};
use locked_voter_state::{Escrow, Locker};
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::RpcFilterType::DataSize;
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use solana_program::pubkey::Pubkey;

pub fn get_delegated_escrows(client: &RpcClient, delegate: &Pubkey) -> Vec<(Pubkey, Escrow)> {
    println!("delegate: {:?}", delegate);
    let accounts = client
        .get_program_accounts_with_config(
            &locked_voter_state::id(),
            RpcProgramAccountsConfig {
                filters: Some(vec![
                    DataSize((ANCHOR_DISCRIMINATOR_SIZE + Escrow::LEN) as u64),
                    RpcFilterType::Memcmp(Memcmp::new(
                        129,
                        MemcmpEncodedBytes::Bytes(delegate.to_bytes().to_vec()),
                    )),
                ]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    commitment: None,
                    data_slice: None,
                    min_context_slot: None,
                },
                with_context: None,
            },
        )
        .unwrap();
    println!("account len: {:?}", accounts.len());
    let mut escrows: Vec<(Pubkey, Escrow)> = Vec::new();
    for (key, account) in accounts {
        if let Ok(parsed_account) = Escrow::try_deserialize(&mut account.data.as_slice()) {
            escrows.push((key, parsed_account));
        }
    }
    escrows
}

struct EpochGuageVoteInfo {
    pub epoch_gauge_vote: Pubkey,
    pub escrow: (Pubkey, Escrow),
}

pub(crate) fn get_escrow_votes(client: &RpcClient, delegate: &Pubkey, gauge: &Pubkey, epoch: u32) {
    let escrows = get_delegated_escrows(client, delegate);
    let mut epoch_gauge_votes: Vec<EpochGuageVoteInfo> = Vec::new();
    let gaugemeister_account = client.get_account(&GAUGEMEISTER).unwrap();
    let gaugemeister_data = Gaugemeister::deserialize(&mut &gaugemeister_account.data[8..]).unwrap();
    let locker_account = client.get_account(&LOCKER).unwrap();
    let locker_data = Locker::deserialize(&mut &locker_account.data[8..]).unwrap();
    for (key, escrow) in escrows.clone() {
        let power = escrow.voting_power_at_time(&locker_data.params, gaugemeister_data.next_epoch_starts_at as i64);
        match power {
            Some(power) => {
                if power < 1000000000 {
                    continue;
                }
            //    println!("account: {:?}, power: {:?}", key, power);
            }
            None => {
                return;
            }
        }
        let vote_accounts = resolve_vote_keys(&key, gauge, epoch);
        let gauge_vote = vote_accounts.epoch_gauge_vote;
        epoch_gauge_votes.push(EpochGuageVoteInfo {
            epoch_gauge_vote: gauge_vote,
            escrow:  (key, escrow)
        });
    }
    println!("number of votes: {:?}", epoch_gauge_votes.len());
    let epoch_gauge_vote_accounts = get_multiple_accounts(client,
      epoch_gauge_votes.iter().map(|x| x.epoch_gauge_vote).collect());
    let mut total_power: u64 = 0;
    for (index, account) in epoch_gauge_vote_accounts.iter().enumerate() {
        let epoch_gauge_vote_data: Option<EpochGaugeVote> = account
            .as_ref()
            .map(|account| EpochGaugeVote::try_deserialize(&mut account.data.as_slice()).unwrap());
        match epoch_gauge_vote_data {
            Some(data) => {
                println!(
                    "account: {:?}, vote: {:?}, escrow: {:?}, epochGaugeVote: {:?}",
                    epoch_gauge_votes[index].escrow.1.owner, data.allocated_power,
                    epoch_gauge_votes[index].escrow.0,
                    epoch_gauge_votes[index].epoch_gauge_vote
                );
                total_power += data.allocated_power;
            }
            None => {
                println!("account: {:?}, Hasn't voted", escrows[index].1.owner);
            }
        }
    }
    println!("total power: {:?}", total_power);
}
