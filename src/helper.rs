pub static GENESIS_PERIOD: i32 = 21600;
pub static BLANK_32_SLICE: [u8; 72] = [0; 72];

pub fn time_since_unix_epoch() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    return since_the_epoch.as_millis();
}

pub fn generate_keys() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    return secp.generate_keypair(&mut thread_rng());
}

// need to implement Hashable trait for Transaction
pub fn create_merkle_root(transactions: Vec<Transaction>) -> Vec<u8> {
    let merkle = MerkleTree::from_vec(&SHA256, transactions);
    let merkle_root: Vec<u8> = merkle.root_hash().clone();
    return merkle_root;
}