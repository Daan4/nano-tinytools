use serde_derive::Deserialize;
use std::fs;
use toml;
use regex::Regex;
use lazy_static::lazy_static;
use nano_tinytools_common::{derive_private_key, derive_public_key, derive_address, hexstring_to_bytes, HEX};

lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

static CONFIG_PATH: &str = "config.toml";

#[derive(Deserialize)]
pub struct Config {
    pub seed: String,
    pub target: String,
    pub start_index: u32,
    pub stop_index: u32,
}

impl Config { 
    fn new() -> Self {
        let contents = fs::read_to_string(CONFIG_PATH).expect("Something went wrong reading the file");
        toml::from_str(&contents).unwrap()
    }
}

fn main() {
    // Read config
    let seed = CONFIG.seed.to_owned();
    let target = CONFIG.target.to_owned();
    let mut index = CONFIG.start_index;
    let stop_index = CONFIG.stop_index;

    // Validate seed format
    assert!(seed.len() == 64);
    assert!(seed.chars().all(|x| HEX.contains(&&x.to_string().as_str())));
    let seed = hexstring_to_bytes(&seed);

    // Validate target format
    let re = Regex::new(r"^(nano|xrb)_[13]{1}[13456789abcdefghijkmnopqrstuwxyz]{59}$").unwrap();
    assert!(re.is_match(&target));

    // Sweep
    while index < stop_index {
        let private_key = derive_private_key(seed, index);
        let public_key = derive_public_key(private_key);
        let address = derive_address(public_key);

        if address == target {
            println!("found {} @ index {}", address, index);
            break;
        }

        index += 1;

        if index % 1000 == 0 {
            println!("{}", index)
        }
    }
}
