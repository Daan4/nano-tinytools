use serde_derive::Deserialize;
use std::fs;
use std::time::{Instant, Duration};
use toml;
use lazy_static::lazy_static;
use nano_tinytools_common::{derive_private_key, derive_public_key, derive_address, hexstring_to_bytes, validate_seed, validate_address};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use crossbeam_deque::{Stealer, Steal, Worker};

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
    let seed = CONFIG.seed.to_owned();
    let target = CONFIG.target.to_owned();
    let num_threads = CONFIG.num_threads;
    let start_index = CONFIG.start_index;
    let stop_index = CONFIG.stop_index;

    // Validate seed and target format
    if !validate_seed(&seed) {
        panic!("Seed must consist of 64 hex characters")
    }
    let seed = hexstring_to_bytes(&seed);

    if !validate_address(&target) {
        panic!("Target must be a valid nano address")
    }

    let q = Worker::new_fifo();

    let (tx, rx): (Sender<()>, Receiver<()>) = channel();
    for id in 0..num_threads {
        let tx_clone = tx.clone();
        let s = q.stealer();
        thread::spawn(move || {
            consumer(id, tx_clone, s, seed);
        });
    }

    thread::spawn(move || {
        producer(q, start_index, stop_index);
    });

    rx.recv().unwrap();
    println!("\ntime taken: {} s", now.elapsed().as_millis() as f32 / 1000.0);
}

fn producer(queue: Worker<u32>, start_index: u32, stop_index: u32) {
    for index in start_index..stop_index {
        queue.push(index);
        if queue.len() > 10000 {
            thread::sleep(Duration::new(0, 100000000));
        }
    }
}

fn consumer(id: u32, tx: Sender<()>, queue: Stealer<u32>, seed: [u8; 32]) {
    let target = CONFIG.target.to_owned();
    let mut count = 0;
    loop {
        match queue.steal() {
            Steal::Success(index) => {
                let private_key = derive_private_key(seed, index);
                let public_key = derive_public_key(private_key);
                let address = derive_address(public_key);
        
                if address == target {
                    println!("\nfound target {} @ index {}", address, index);
                    tx.send(()).unwrap();
                    break;
                }
        
                count += 1;
                if count % 1000 == 0 {
                    println!("thread {} count {}", id, count);
                }
            },
            _ => {}
        }
    }
}
