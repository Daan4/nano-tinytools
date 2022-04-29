use serde_derive::Deserialize;
use std::fs;
use std::time::Instant;
use toml;
use lazy_static::lazy_static;
use nano_tinytools_common::{derive_private_key, derive_public_key, derive_address, hexstring_to_bytes, validate_seed, validate_address};

lazy_static! {
    static ref CONFIG: Config = Config::new();
}

static CONFIG_PATH: &str = "config.toml";

#[derive(Deserialize)]
struct Config {
    seed: String,
    target: String,
    start_index: u32,
    stop_index: u32,
}

impl Config { 
    fn new() -> Self {
        let contents = fs::read_to_string(CONFIG_PATH).expect("Something went wrong reading the file");
        toml::from_str(&contents).unwrap()
    }
}

fn main() {
    let now = Instant::now();

    // Read config
    let seed = CONFIG.seed.to_owned();
    let target = CONFIG.target.to_owned();
    let mut index = CONFIG.start_index;
    let stop_index = CONFIG.stop_index;

    // Validate seed and target format
    if !validate_seed(&seed) {
        panic!("Seed must consist of 64 hex characters")
    }
    let seed = hexstring_to_bytes(&seed);

    if !validate_address(&target) {
        panic!("Target must be a valid nano address")
    }

    // Sweep
    while index < stop_index {
        let private_key = derive_private_key(seed, index);
        let public_key = derive_public_key(private_key);
        let address = derive_address(public_key);

        if address == target {
            println!("\nfound target {} @ index {}", address, index);
            break;
        }

        index += 1;

        if index % 1000 == 0 {
            println!("{}", index)
        }
    }

    println!("\ntime taken: {} s", now.elapsed().as_millis() as f32 / 1000.0);
}
