use crate::accounts::resolve::{get_delegate, resolve_vote_keys};
use crate::{GAUGEMEISTER, LOCKER};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::Keypair;

pub struct WeightInfo {
    pub gauge: Pubkey,
    pub weight: u32,
}

pub fn vote(
    anchor_client: &anchor_client::Client<&Keypair>,
    client: &RpcClient,
    script_authority: &Keypair,
    config: Pubkey,
    escrow: Pubkey,
    epoch: u32,
    weights: Vec<WeightInfo>,
) {
    let vote_delegate = get_delegate(&config);
    let program = anchor_client.program(vote_market::id()).unwrap();

    // Set weights
    for weight in weights {
        // Set weight
        let vote_accounts = resolve_vote_keys(&escrow, &weight.gauge, epoch);
        println!("Epoch the votes are for: {}", epoch);

        let _sig = program
            .request()
            .signer(script_authority)
            .args(vote_market::instruction::Vote {
                weight: weight.weight,
            })
            .accounts(vote_market::accounts::Vote {
                config,
                script_authority: script_authority.pubkey(),
                gaugemeister: GAUGEMEISTER,
                gauge: weight.gauge,
                gauge_voter: vote_accounts.gauge_voter,
                gauge_vote: vote_accounts.gauge_vote,
                escrow,
                vote_delegate,
                gauge_program: gauge_state::id(),
            })
            .send()
            .unwrap();

        let data: Vec<u8> = solana_program::hash::hash(b"global:prepare_epoch_gauge_voter_v2")
            .to_bytes()[..8]
            .to_vec();
        let create_epoch_gauge_voter_ix = solana_program::instruction::Instruction {
            program_id: gauge_state::id(),
            accounts: vec![
                //Gauge vote account
                AccountMeta::new_readonly(GAUGEMEISTER, false),
                AccountMeta::new_readonly(LOCKER, false),
                AccountMeta::new_readonly(escrow, false),
                AccountMeta::new_readonly(vote_accounts.gauge_voter, false),
                AccountMeta::new(vote_accounts.epoch_gauge_voter, false),
                AccountMeta::new(script_authority.pubkey(), true),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
            ],
            data,
        };
        let mut transaction = solana_sdk::transaction::Transaction::new_with_payer(
            &[create_epoch_gauge_voter_ix],
            Some(&script_authority.pubkey()),
        );
        let latest_blockhash = client.get_latest_blockhash().unwrap();
        transaction.sign(&[script_authority], latest_blockhash);
        let result = client.send_and_confirm_transaction(&transaction).unwrap();
        println!("prepare epoch gauge voter result: {:?}", result);
        println!("transaction: {:?}", transaction.signatures.first().unwrap());
        // Commit vote

        let mut data: Vec<u8> =
            solana_program::hash::hash(b"global:gauge_commit_vote_v2").to_bytes()[..8].to_vec();
        data.extend_from_slice(&weight.weight.to_le_bytes());
        let commit_vote_ix = solana_program::instruction::Instruction {
            program_id: gauge_state::id(),
            accounts: vec![
                AccountMeta::new_readonly(GAUGEMEISTER, false),
                AccountMeta::new_readonly(weight.gauge, false),
                AccountMeta::new_readonly(vote_accounts.gauge_voter, false),
                AccountMeta::new_readonly(vote_accounts.gauge_vote, false),
                AccountMeta::new(vote_accounts.epoch_gauge, false),
                AccountMeta::new(vote_accounts.epoch_gauge_voter, false),
                AccountMeta::new(vote_accounts.epoch_gauge_vote, false),
                AccountMeta::new(script_authority.pubkey(), true),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
            ],
            data,
        };
        let mut transaction = solana_sdk::transaction::Transaction::new_with_payer(
            &[commit_vote_ix],
            Some(&script_authority.pubkey()),
        );
        let latest_blockhash = client.get_latest_blockhash().unwrap();
        transaction.sign(&[script_authority], latest_blockhash);
        let result = client
            .send_and_confirm_transaction_with_spinner_and_config(
                &transaction,
                CommitmentConfig::confirmed(),
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    ..RpcSendTransactionConfig::default()
                },
            )
            .unwrap();
        println!("Vote committed {}", result);
    }
}
