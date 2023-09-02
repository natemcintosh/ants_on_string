use fastrand;

/// Simulate a single fast run. Simulate placing `n_ants` on the string, but only
/// actually keep track of the ones at the top and bottom. Then pick random directions
/// for those two ants, and calculate the max time. `ant_vel_mps` is the ant velocity in
/// meters/second
fn fast_ant_sim(n_ants: usize, ant_vel_mps: f64) -> f64 {
    std::iter::repeat_with(|| fastrand::f64())
        // Simulate n_ants
        .take(n_ants)
        // Calculate the distance for each ant
        .map(|pos| {
            // If true, then facing down, and travel from their current position to 0.0
            if fastrand::bool() {
                pos / ant_vel_mps
            } else {
                // Otherwise facing up, and have to travel from current position to 1.0
                (1.0 - pos) / ant_vel_mps
            }
        })
        // Return the max
        .max_by(|a, b| a.partial_cmp(b).expect("Found a NaN in the ant positions"))
        .expect("No ants were simulated")
}

fn main() {
    let n_sims = 1_000;
    let n_ants = 10_000;

    let start_time = std::time::Instant::now();

    let times: Vec<f64> = (0..n_sims)
        .into_iter()
        .map(|_| fast_ant_sim(n_ants, 0.01))
        .collect();

    let run_time = start_time.elapsed();
    let secs_sim_time = run_time.as_micros() / (n_sims as u128);

    let mean: f64 = times.iter().sum::<f64>() / (times.len() as f64);
    println!("Mean time over {n_sims} sims for {n_ants} ants was {mean:0.2}s");
    println!("Average sim runtime was {secs_sim_time}us");
}
