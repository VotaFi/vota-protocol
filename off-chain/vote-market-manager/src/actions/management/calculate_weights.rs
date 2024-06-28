use crate::actions::management::data::{EpochData, VoteInfo};

pub(crate) fn calculate_weights(
    data: &mut EpochData,
) -> Result<Vec<VoteInfo>, Box<dyn std::error::Error>> {
    let total: f64 = data.gauges.iter().map(|g| g.payment).sum();
    println!("multiplier 1 {}", u32::MAX as f64 / total);
    let multiplier = (u32::MAX - 100) as f64 / total;
    println!("multiplier 2 {}", multiplier);
    let mut vote_infos: Vec<VoteInfo> = Vec::new();
    println!("delegated votes: {}", data.delegated_votes);
    for gauge in &data.gauges {
        println!("gauges payment: {:?}", gauge.payment);
        let vote_info = VoteInfo {
            weight: (gauge.payment * multiplier) as u32,
            votes: ((data.delegated_votes as f64) * (gauge.payment / total)) as u64,
            gauge: gauge.gauge,
        };
        vote_infos.push(vote_info);
    }
    Ok(vote_infos)
}
