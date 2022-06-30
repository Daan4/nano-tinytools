use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;
use regex::Regex;
use std::fs;
use toml;
use serde_derive::Deserialize;
use lazy_static::lazy_static;
use nano_tinytools_common::{derive_private_key, derive_public_key, derive_address, bytes_to_hexstring, generate_random_seed};

lazy_static! {
    static ref CONFIG: Config = Config::new();
}

static CONFIG_PATH: &str = "config.toml";

#[derive(Deserialize)]
struct Config {
    starts_with: String,
    ends_with: String,
    contains: String,
    num_threads: u32,
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
    let starts_with = CONFIG.starts_with.as_str();
    let ends_with = CONFIG.ends_with.as_str();
    let contains = CONFIG.contains.as_str();
    let num_threads = CONFIG.num_threads;

    // Validate config
    let re = Regex::new(r"[13456789abcdefghijkmnopqrstuwxyz]{0,59}$").unwrap();
    if starts_with != "" && !re.is_match(starts_with) {
        println!("Invalid starts_with setting");
        return;
    }
    if ends_with != "" && !re.is_match(ends_with) {
        println!("Invalid ends_with setting");
        return;
    }
    if contains != "" && !re.is_match(contains) {
        println!("Invalid contains setting");
        return;
    }

    let (tx, rx): (Sender<()>, Receiver<()>) = channel();

    for id in 0..num_threads {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            inner(id, tx_clone);
        });
    }

    rx.recv().unwrap();
    println!("\ntime taken: {} s", now.elapsed().as_millis() as f32 / 1000.0);
}

fn inner(id: u32, tx: Sender<()>) {
    let starts_with = CONFIG.starts_with.as_str();
    let ends_with = CONFIG.ends_with.as_str();
    let contains = CONFIG.contains.as_str();
    let mut count = 0;

    loop {
        let seed = generate_random_seed();
        let private_key = derive_private_key(seed, 0);
        let public_key = derive_public_key(private_key);
        let address = derive_address(public_key);
        if address[1..].starts_with(starts_with)
            && address.ends_with(ends_with)
            && address.contains(contains)
        {
            println!("\naddress: {}\nseed: {}", address, bytes_to_hexstring(&seed));
            tx.send(()).unwrap();
            break;
        }

        count += 1;

        if count % 1000 == 0 {
            println!("thread {} count {}", id, count);
        }
    }
}
