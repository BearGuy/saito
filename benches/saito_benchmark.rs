#[macro_use]
extern crate criterion;
extern crate bincode;
extern crate saito;

use std::time::{SystemTime, UNIX_EPOCH, Duration};

use bincode::{serialize_into, deserialize_from};
use std::fs::{File, read_dir};
use std::io::{BufWriter, BufReader, Read};
use std::io::prelude::*;
use std::path::Path;

use criterion::{Criterion, Benchmark};

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

    // for x in 0..10 {
        let mut tx = Transaction::new(TransactionType::Base);
        tx.msg =  (0..1073741824).map(|_| { rand::random::<u8>() }).collect();
        block.transactions.push(tx);
    // }

    return block;
}

fn write_blocks(blocks: &Vec<Block>) {
    for block in blocks.iter() {
        // let mut filename = "data/".to_string();
        // filename.push_str(&time_since_unix_epoch().to_string());
        // filename.push_str(&".sai".to_string());
        let filename = Path::new("blockdata.sai");

        // let mut f = BufWriter::new(File::create(filename).unwrap());
        // serialize_into(&mut f, block).unwrap();

        let encode: Vec<u8> = bincode::serialize(block).unwrap();
        // let filename = Path::new("blockdata.sai");
        let mut f = File::create(filename).unwrap();
        f.write_all(&encode[..]);
    }
}

fn read_blocks() {
    let dir = Path::new("data");
    if dir.is_dir() {
        for entry in read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            let mut encoded = Vec::<u8>::new();
            let mut r = File::open(&path).unwrap();
            r.read_to_end(&mut encoded).unwrap();
            let decoded: Block = bincode::deserialize(&encoded[..]).unwrap();
        }
    }
}

fn read_block() {
    // let entry = entry.unwrap();
    // let path = entry.path();
    let filename = Path::new("blockdata.sai");
    let mut encoded = Vec::<u8>::new();

    let mut r = File::open(&filename).unwrap();
    r.read_to_end(&mut encoded).unwrap();
    let decoded: Block = bincode::deserialize(&encoded[..]).unwrap();
}

fn serialize_and_deserialize_with_file(block: &Block) {
    let encode: Vec<u8> = bincode::serialize(block).unwrap();
    let filename = Path::new("blockdata.sai");
    let mut f = File::create(filename).unwrap();
    f.write_all(&encode[..]);
    // let mut f = BufWriter::new(File::create(filename).unwrap());
    // serialize_into(&mut f, block).unwrap();

    let mut r = File::open(&filename).unwrap();
    let mut encoded = Vec::<u8>::new();
    r.read_to_end(&mut encoded).unwrap();

    let decoded: Block = bincode::deserialize(&encoded[..]).unwrap();
    // let mut r = BufReader::new(File::open(&filename).unwrap());
    // let r = BufReader::new(&encoded[..]);
    // let deserialized_block: Block = deserialize_from(&mut r).unwrap();
    // assert_eq!(block, &decoded);
}

fn serialize_and_deserialize_in_memory(block: &Block) {
    let encoded: Vec<u8> = bincode::serialize(block).unwrap();
    let decoded: Block = bincode::deserialize(&encoded[..]).unwrap();
}

fn serialize_in_memory(block: &Block) {
    let encoded: Vec<u8> = bincode::serialize(block).unwrap();
}

fn create_block_benchmark(c: &mut Criterion) {
    //c.bench_function("serialize blocks and write to disk", move |b| b.iter(|| write_blocks(&blocks)));

    c.bench(
        "create block data in memory",
        Benchmark::new(
            "encode",
            move |b| b.iter(|| create_block()),
        )
        .sample_size(2)
    );
}

fn serialize_blocks_benchmark(c: &mut Criterion) {
    // let (secret_key, public_key) = generate_keys();
    let mut blocks: Vec<Block> = Vec::new();

    // for x in 0..100 {
    blocks.push(create_block());
    // }

    c.bench(
        "serialize blocks to disk",
        Benchmark::new(
            "encode",
            move |b| b.iter(|| write_blocks(&blocks))
        )
        .sample_size(2)
    );
}

fn deserialize_blocks_benchmark(c: &mut Criterion) {
    c.bench(
        "deserialize blocks from disk",
        Benchmark::new(
            "decode",
            move |b| b.iter(|| read_blocks()),
        )
        .sample_size(2)
    );
}


fn deserialize_block_from_disk_benchmark(c: &mut Criterion) {
    c.bench(
        "deserialize blocks from disk",
        Benchmark::new(
            "decode",
            move |b| b.iter(|| read_block()),
        )
        .sample_size(2)
    );
}

fn serialize_in_memory_benchmark(c: &mut Criterion) {
    let block = create_block();
    c.bench(
        "serialize a block in memory",
        Benchmark::new(
            "serialize/memory",
            move |b| b.iter(|| serialize_in_memory(&block)),
        )
        .sample_size(10)
    );
}

fn serialize_deserialize_in_memory_benchmark(c: &mut Criterion) {
    let block = create_block();
    c.bench(
        "serialize and deserialize block in memory",
        Benchmark::new(
            "memory",
            move |b| b.iter(|| serialize_and_deserialize_in_memory(&block)),
        )
        .sample_size(2)
    );
}

fn serialize_deserialize_to_file_benchmark(c: &mut Criterion) {
    let block = create_block();
    c.bench(
        "serialize and deserialize block from disk",
        Benchmark::new(
            "file",
            move |b| b.iter(|| serialize_and_deserialize_with_file(&block))
        )
        .sample_size(2)
    );
}

// criterion_group!(benches, deserialize_blocks_benchmark);
//criterion_group!(benches, create_block_benchmark, serialize_blocks_benchmark, deserialize_blocks_benchmark, serialize_deserialize_to_file_benchmark);
criterion_group!(benches, deserialize_block_from_disk_benchmark);
criterion_main!(benches);

