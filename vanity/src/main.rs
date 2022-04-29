use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;
use regex::Regex;
use nano_tinytools_common::{derive_private_key, derive_public_key, derive_address, bytes_to_hexstring};

// --SETTINGS--

// The text the vanity address should start with, empty to ignore
const STARTS_WITH: &str = "";

// The text the vanity address should end with, empty to ignore
const ENDS_WITH: &str = "";

// The text the vanity address should contain, empty to ignore
const CONTAINS: &str = "daan";

// Number of threads, set to number of logical processors
const NUM_THREADS: u32 = 4;

fn main() {
    // benchmark();

    let re = Regex::new(r"[13456789abcdefghijkmnopqrstuwxyz]{0,59}$").unwrap();
    if STARTS_WITH != "" && !re.is_match(STARTS_WITH) {
        println!("Invalid STARTS_WITH setting");
        return;
    }
    if ENDS_WITH != "" && !re.is_match(ENDS_WITH) {
        println!("Invalid ENDS_WITH setting");
        return;
    }
    if CONTAINS != "" && !re.is_match(CONTAINS) {
        println!("Invalid CONTAINS setting");
        return;
    }

    let mut ts: Vec<thread::JoinHandle<()>> = vec![];
    let (tx, rx): (Sender<()>, Receiver<()>) = channel();

    for _ in 0..NUM_THREADS {
        let tx_clone = tx.clone();
        ts.push(thread::spawn(|| {
            let rng = &mut ChaCha20Rng::from_entropy();
            inner(&mut rng.clone(), tx_clone);
        }));
    }

    rx.recv().unwrap();
}

fn inner(rng: &mut ChaCha20Rng, tx: Sender<()>) {
    loop {
        let seed = generate_random_seed(rng);
        let private_key = derive_private_key(seed, 0);
        let public_key = derive_public_key(private_key);
        let address = derive_address(public_key);
        if address[1..].starts_with(STARTS_WITH)
            && address.ends_with(ENDS_WITH)
            && address.contains(CONTAINS)
        {
            println!("{}\n{}", address, bytes_to_hexstring(&seed));
            tx.send(()).unwrap();
            break;
        }
    }
}

fn benchmark() {
    let rng = &mut ChaCha20Rng::from_entropy();

    let mut count = 0;
    let runs = 20000;
    let now = Instant::now();
    while count < runs {
        let seed = generate_random_seed(rng);
        let private_key = derive_private_key(seed, 0);
        let public_key = derive_public_key(private_key);
        derive_address(public_key);
        count += 1;
    }
    println!(
        "{} runs: {}s",
        runs,
        now.elapsed().as_millis() as f32 / 1000.0
    );
}

/// Generate random seed
fn generate_random_seed(rng: &mut ChaCha20Rng) -> [u8; 32] {
    let mut seed = [0; 32];
    for i in 0..32 {
        seed[i] = rng.gen_range(0..16) << 4 | rng.gen_range(0..16);
    }
    seed
}
