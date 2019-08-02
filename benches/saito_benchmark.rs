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

// multi-threading
use std::thread;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use rayon::prelude::*;

use criterion::{Criterion, Benchmark};

use secp256k1::{Secp256k1, SecretKey, PublicKey};
use rand::Rng;
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

fn create_block(public_key: PublicKey) -> Block {
    // let (secret_key, public_key) = generate_keys();
    let mut block = Block::new(Vec::new(), public_key);
    //let mut rng = rand::thread_rng();

    for x in 0..100 {
        let mut tx = Transaction::new(TransactionType::Base);
        //tx.msg = (0..5073741).map(|_| { rng.gen(); }).collect();
        tx.msg = (0..5073741).map(|_| { rand::random::<u8>() }).collect();
        block.transactions.borrow_mut().push(tx);
    }

    return block;
}

fn create_block_multi(public_key: PublicKey) -> Block {
    let mut block = Block::new(Vec::new(), public_key);

    let (sender, receiver) = channel();

    (0..1000000).into_par_iter().for_each_with(sender, |s, x| create_transaction_multi(s.clone()));
    block.transactions.replace(receiver.iter().collect());
    return block;
}


fn create_transaction_multi(sender: Sender<Transaction>) {
    let mut tx = Transaction::new(TransactionType::Base);
    // let mut rng = thread_rng();
    //tx.msg = (0..5073741).map(|_| { rand::random::<u8>() }).collect();
    tx.msg = (0..5073)
        .into_par_iter()
        .map_init(
            || rand::thread_rng(),
            |rng, x| rng.gen()
        ).collect();
    sender.send(tx).unwrap();
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

fn generate_mhash(block: &Block) {
    block.transactions
        .borrow()
        .clone()
        .into_par_iter()
        .map(|tx| tx.return_message_hash());
}

//
// Benchmarks
//

fn create_block_benchmark(c: &mut Criterion) {
    //c.bench_function("serialize blocks and write to disk", move |b| b.iter(|| write_blocks(&blocks)));
    let (secret_key, public_key) = generate_keys();
    c.bench(
        "create block data in memory",
        Benchmark::new(
            "encode",
            move |b| b.iter(|| create_block(public_key)),
        )
        .sample_size(2)
    );
}

fn create_block_multi_thread_benchmark(c: &mut Criterion) {
    let (secret_key, public_key) = generate_keys();
    c.bench(
        "multi thread block",
        Benchmark::new(
            "multi thread transaction",
            move |b| b.iter(|| create_block_multi(public_key))
        )
        .sample_size(2)
    );
}


fn serialize_blocks_benchmark(c: &mut Criterion) {
    let (secret_key, public_key) = generate_keys();
    let mut blocks: Vec<Block> = Vec::new();

    // for x in 0..100 {
    blocks.push(create_block(public_key));
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
    let (secret_key, public_key) = generate_keys();
    let block = create_block(public_key);
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
    let (secret_key, public_key) = generate_keys();
    let block = create_block(public_key);
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
    let (secret_key, public_key) = generate_keys();
    let block = create_block(public_key);
    c.bench(
        "serialize and deserialize block from disk",
        Benchmark::new(
            "file",
            move |b| b.iter(|| serialize_and_deserialize_with_file(&block))
        )
        .sample_size(2)
    );
}

fn generate_mhash_benchmark(c: &mut Criterion) {
    let (secret_key, public_key) = generate_keys();
    let block = create_block_multi(public_key);
    c.bench(
        "generate mhashes for all tx in block",
        Benchmark::new(
            "hash",
            move |b| b.iter(|| generate_mhash(&block))
        )
        .sample_size(2)
    );
}

// criterion_group!(benches, deserialize_blocks_benchmark);
//criterion_group!(benches, create_block_benchmark, serialize_blocks_benchmark, deserialize_blocks_benchmark, serialize_deserialize_to_file_benchmark);
//criterion_group!(benches, multi_thread_transactions_benchmark);
// criterion_group!(benches, create_block_multi_thread_benchmark);
criterion_group!(benches, generate_mhash_benchmark);
criterion_main!(benches);

