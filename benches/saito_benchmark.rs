#[macro_use]
extern crate criterion;
extern crate bincode;
extern crate saito;

use std::time::{SystemTime, UNIX_EPOCH};

use bincode::{serialize_into, deserialize_from};
use std::fs::{File, read_dir};
use std::io::{BufWriter, BufReader};
use std::path::Path;

use criterion::Criterion;

use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::thread_rng;

use saito::{Block, Transaction, TransactionType};

fn time_since_unix_epoch() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    return since_the_epoch.as_millis();
}

fn generate_keys() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    return secp.generate_keypair(&mut thread_rng());
}

fn create_block() -> Block {
    let (secret_key, public_key) = generate_keys();
    let mut block = Block::new(Vec::new(), public_key);
    let mut tx = Transaction::new(TransactionType::Base);
    tx.msg =  (0..1024).map(|_| { rand::random::<u8>() }).collect();
    block.transactions.push(tx);
    return block;
}

fn write_blocks(blocks: &Vec<Block>) {
    for block in blocks.iter() {
        let mut filename = "data/".to_string();
        filename.push_str(&time_since_unix_epoch().to_string());
        filename.push_str(&".sai".to_string());

        let mut f = BufWriter::new(File::create(filename).unwrap());
        serialize_into(&mut f, block).unwrap();
    }
}

fn read_blocks() {
    let dir = Path::new("data");
    if dir.is_dir() {
        for entry in read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let mut r = BufReader::new(File::open(path).unwrap());
            let block: Block = deserialize_from(&mut r).unwrap();
        }
    }
}

fn serialize_and_deserialize(block: &Block) {
    // let encoded: Vec<u8> = bincode::serialize(block).unwrap();
    let filename = "blockdata.sai";
    let mut f = BufWriter::new(File::create(filename).unwrap());
    serialize_into(&mut f, block).unwrap();

    // let decoded: Block = bincode::deserialize(&encoded[..]).unwrap();
    //let mut r = BufReader::new(
    let mut r = File::open(&filename).unwrap();
    let deserialized_block: Block = deserialize_from(&mut r).unwrap();
    // assert_eq!(block, &decoded);
}

fn serialize_blocks_benchmark(c: &mut Criterion) {
    let (secret_key, public_key) = generate_keys();
    let mut blocks: Vec<Block> = Vec::new();

    // for x in 0..100 {
        blocks.push(create_block());
    // }


    c.bench_function("serialize blocks to disk", move |b| b.iter(|| write_blocks(&blocks)));
}

fn deserialize_blocks_benchmark(c: &mut Criterion) {
    c.bench_function("deserialize blocks to memory", move |b| b.iter(|| read_blocks()));
}

fn serialize_deserialize_benchmark(c: &mut Criterion) {
    let block = create_block();

    c.bench_function("serialize then deserialize blocks", move |b| b.iter(|| serialize_and_deserialize(&block)));
}

// criterion_group!(benches, serialize_blocks_benchmark, deserialize_blocks_benchmark);
criterion_group!(benches, serialize_deserialize_benchmark);
criterion_main!(benches);

