use crate::ANCHOR_DISCRIMINATOR_SIZE;
use anchor_lang::AnchorDeserialize;
use gauge_state::EpochGauge;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};

pub(crate) fn get_direct_votes(
    client: &RpcClient,
    epoch: u32,
) -> Result<Vec<EpochGauge>, Box<dyn std::error::Error>> {
    let accounts = client.get_program_accounts_with_config(
        &gauge_state::id(),
        RpcProgramAccountsConfig {
            filters: Some(vec![
                // DataSize((ANCHOR_DISCRIMINATOR_SIZE + EpochGauge::LEN) as u64),
                RpcFilterType::Memcmp(Memcmp::new(
                    ANCHOR_DISCRIMINATOR_SIZE + 32,
                    MemcmpEncodedBytes::Bytes(epoch.to_le_bytes().to_vec()),
                )),
                RpcFilterType::Memcmp(Memcmp::new(
                    0,
                    MemcmpEncodedBytes::Bytes(vec![
                        0x53u8, 0xe5u8, 0x77u8, 0x85u8, 0x7eu8, 0xd1u8, 0x37u8, 0x6eu8,
                    ]),
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
    )?;
    println!("accounts: {:?}", accounts.len());
    accounts
        .iter()
        .map(|(_pubkey, account)| {
            let epoch_guage = EpochGauge::deserialize(&mut &account.data[8..])?;
            Ok(epoch_guage)
        })
        .collect()
}
