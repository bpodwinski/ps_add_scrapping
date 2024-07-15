use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tokio::time::{sleep, Duration};

pub async fn generate_random_delay(min_delay: u64, max_delay: u64) {
    let mut rng = StdRng::from_entropy();
    let delay = rng.gen_range(min_delay..max_delay);

    println!("Delay: {} milliseconds", delay);
    sleep(Duration::from_millis(delay)).await;
}
