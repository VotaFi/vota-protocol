use crate::actions::management::data::EpochData;
use chrono::Utc;
use solana_program::pubkey::Pubkey;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

pub(crate) fn create_parallel_sh(
    data_file: &String,
    weights_file: &String,
    epoch_data: EpochData,
) -> Result<(), Box<dyn std::error::Error>> {
    let escrows: Vec<Pubkey> = epoch_data.escrow_owners.clone();
    let timestamp = Utc::now().format("%Y-%m-%d-%H_%M");
    let output_file = format!("parallel_{}.sh", timestamp);
    let mut file = File::create(&output_file)?;
    let mut permissions = fs::metadata(&output_file)?.permissions();
    // Set the file to be executable by the owner
    permissions.set_mode(0o755);
    // Apply the new permissions to the file
    fs::set_permissions(&output_file, permissions)?;

    writeln!(file, "#!/bin/bash")?;
    writeln!(file)?;
    writeln!(file, "# Define the list of escrow owners")?;
    writeln!(file)?;
    writeln!(file, "accounts=(")?;

    for pubkey in escrows {
        writeln!(file, "\"{}\"", pubkey)?;
    }

    writeln!(file, ")")?;
    writeln!(file)?;
    writeln!(file, "max_parallel_jobs=5")?;
    writeln!(file, "current_jobs=0")?;
    writeln!(file)?;
    writeln!(file, "start_time=$(date +%s)")?;
    writeln!(file, "# Execute the command for each account in parallel")?;
    writeln!(file, "for account in \"${{accounts[@]}}\"")?;
    writeln!(file, "do")?;
    writeln!(file, "    echo \"Processing account: $account\"")?;
    writeln!(
        file,
        "    ./target/release/vote-market-manager execute-votes {} {} -k ~/.config/solana/goki_owner.json -e \"$account\" &",
        data_file, weights_file
    )?;
    writeln!(file, "    current_jobs=$((current_jobs + 1))")?;
    writeln!(file)?;
    writeln!(
        file,
        "    # If the number of current jobs equals the maximum allowed, wait for them to finish"
    )?;
    writeln!(
        file,
        "    if [ \"$current_jobs\" -ge \"$max_parallel_jobs\" ]; then"
    )?;
    writeln!(file, "        wait -n")?;
    writeln!(
        file,
        "        # Decrease the count of current jobs after one completes"
    )?;
    writeln!(file, "        current_jobs=$((current_jobs - 1))")?;
    writeln!(file, "    fi")?;
    writeln!(file, "done")?;
    writeln!(file)?;
    writeln!(file, "# Wait for all background processes to finish")?;
    writeln!(file, "wait")?;
    writeln!(file)?;
    writeln!(file, "end_time=$(date +%s)")?;
    writeln!(file)?;
    writeln!(file, "elapsed_time=$((end_time - start_time))")?;
    writeln!(file)?;
    writeln!(
        file,
        "echo \"All processes completed in $elapsed_time seconds.\""
    )?;
    Ok(())
}
