extern crate sha2;
extern crate rand;
extern crate base58;
extern crate secp256k1;
extern crate merkle;
extern crate serde;

use std::time::{SystemTime, UNIX_EPOCH};
//use std::{thread, time};
use std::cell::{RefCell, RefMut};
use std::mem::transmute;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde::ser::{Serializer, SerializeStruct};
//use serde::de::{self, Deserializer, Visitor, SeqAccess, MapAccess};

//use std::fmt;

use sha2::Sha256;
use digest::Digest;

use ring::digest::{SHA256, Context};

use secp256k1::{Secp256k1, Message, Signature};
use secp256k1::{SecretKey, PublicKey};

use merkle::{MerkleTree, Hashable};

use rand::{Rng,thread_rng};

use base58::{ToBase58};
//use byteorder::{BigEndian, ReadBytesExt};

static GENESIS_PERIOD: i32 = 21600;
static BLANK_32_SLICE: [u8; 72] = [0; 72];

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

// need to implement Hashable trait for Transaction
pub fn create_merkle_root(transactions: Vec<Transaction>) -> Vec<u8> {
    let merkle = MerkleTree::from_vec(&SHA256, transactions);
    let merkle_root: Vec<u8> = merkle.root_hash().clone();
    return merkle_root;
}


#[derive(Debug)]
pub struct Miner {
    is_mining: bool,
    can_i_mine: bool,
    difficulty: f32,
    paysplit: f32,
}

impl Miner {
    pub fn new() -> Miner {
        return Miner {
            is_mining: false,
            can_i_mine: false,
            difficulty: 2.0,
            paysplit: 0.5,
        };
    }

    pub fn start_mining(&mut self,
                    mempool: &RefCell<Mempool>,
                    previous_block: &Block,
                    wallet: &Wallet,
                    burnfee: &BurnFee) {
        self.can_i_mine = true;
        self.is_mining = true;

        while self.can_i_mine {
            self.attempt_solution(mempool, previous_block, wallet, burnfee);
        };
    }

    pub fn stop_mining(&mut self) {
        self.can_i_mine = false;
    }

    // need previous_block, previous_block: &Block
    fn attempt_solution(&mut self,
                        mempool: &RefCell<Mempool>,
                        previous_block: &Block,
                        wallet: &Wallet,
                        burnfee: &BurnFee) {

        let mut rng = thread_rng();
        let random_number = rng.gen::<u32>();
        let random_number_bytes: [u8; 4] = unsafe { transmute(random_number.to_be()) };

        // hash our solution
        let mut hasher = Sha256::new();
        let publickey_vec: Vec<u8> = wallet.publickey.serialize().iter().cloned().collect();
        hasher.input(publickey_vec);
        hasher.input(random_number_bytes);

        let result = hasher.clone().result();
        let random_solution_slice = result.as_slice();
        let random_solution_vec = hasher.result().to_vec();

        if self.is_valid_solution(&random_solution_vec, &previous_block.return_block_hash()) {
            // Stop mining
            //hasher.result().as_slice()
            println!("WE HAVE FOUND A SOLUTION");
            self.can_i_mine = false;

            let golden_tx_solution = self.calculate_solution(
                wallet.publickey,
                &previous_block.return_block_hash(),
                &random_solution_vec
            );

            // Find winning node
            let winning_tx_address = self.find_winner(&random_solution_slice, &previous_block);

            // we need to calculate the fees that are gonna go in the slips here
            let paid_burn_fee = burnfee.return_previous_burnfee();

            // This is just inputs - outputs for all transactions in the block
            let total_fees_for_creator = previous_block.return_available_fees(&previous_block.creator);

            // get the fees available from our publickey
            let total_fees_in_block = previous_block.return_available_fees(&wallet.publickey);

            // calculate the amount the creator can take for themselves
            let creator_surplus = total_fees_for_creator - paid_burn_fee;

            // find the amount that will be divied out to miners and nodes
            let total_fees_for_miners_and_nodes = (total_fees_in_block - creator_surplus) + previous_block.coinbase;

            // Calculate Shares
            let miner_share = total_fees_for_miners_and_nodes * self.paysplit;
            let node_share  = total_fees_for_miners_and_nodes - miner_share;

            println!("CREAINTG GOLDEN TX");
            let mut golden_tx = Transaction::new(TransactionType::GoldenTicket);

            golden_tx.add_to_slip(Slip {
                address: wallet.publickey,
                amount: miner_share,
                block_id: 0,
                transaction_id: 0,
                id: 0,
                block_hash: Vec::new(),
                lc: 1
            });

            golden_tx.add_to_slip(Slip {
                address: winning_tx_address,
                amount: node_share,
                block_id: 0,
                transaction_id: 0,
                id: 0,
                block_hash: Vec::new(),
                lc: 1
            });

            golden_tx.add_from_slip(Slip {
                address: wallet.publickey,
                amount: 0.0,
                block_id: 0,
                transaction_id: 0,
                id: 0,
                block_hash: Vec::new(),
                lc: 1
            });

            // sign TX
            println!("CREATING SIGNATURE");
            golden_tx.sig = wallet.create_signature(golden_tx.return_signature_source().as_slice());

            mempool.borrow_mut().add_transaction(golden_tx);
        }
    }

    fn calculate_solution(&self, publickey: PublicKey, block_hash: &Vec<u8>, random: &Vec<u8>) -> GoldenTicket {
        let mut vote: u8 = 0;
//        if block.difficulty > self.difficulty {
//            vote = 1;
//        }
        return GoldenTicket {
            target: block_hash.clone(),
            vote: 1,
            random: random.clone(),
            publickey,
        }
    }

    fn is_valid_solution(&self, random_solution: &Vec<u8>, prev_blk: &Vec<u8>) -> bool {

        // static difficulty until this is implemented on the block object
        let difficulty = self.difficulty.round() as usize;

        let random_solution_slice = &random_solution[0..difficulty];
        let previous_hash_slice = &prev_blk[0..difficulty];

        //println!("RANDOM SOLUTION {}", random_solution_slice);
        //println!("PREVIOUS HASH SLICE {}", random_solution_slice);

        if random_solution_slice == previous_hash_slice {
            return true
        } else {
            return false;
        }
    }

    fn find_winner(&self, random_solution: &[u8], previous_block: &Block) -> PublicKey {
       // let max_hash = 0xFFFFFFFF;

       let winning_address: PublicKey;
       if previous_block.transactions.len() == 0 { return previous_block.creator; }

       //let random_slice = random_solution;

       //let max_num: u32 = u32::from_str_radix("ffffffffffff", 16).unwrap();
       //let win_num: u32 = unsafe { transmute(random_solution.to_be()) };
       //let win_num = random_solution.clone().read_u32::<BigEndian>().unwrap(); //(random_solution);

       //let winner_dec = win_num as f32 / max_num as f32;

       let mut winning_tx = self.find_winning_transaction(previous_block);

       // until we differentiate between fees and amount, we'll just use amount

       winning_address = match winning_tx {
          Some(tx) => tx.from[0].address,
          None => previous_block.creator
       };

       return winning_address;

       //return winning_tx.from[0].address;
    }

    fn find_winning_transaction(&self, previous_block: &Block) -> Option<Transaction> {
        let mut winning_tx = Transaction::new(TransactionType::Base);
        let mut winning_amt = 0.0;
        for tx in previous_block.transactions.iter() {
            let current_amt = tx.calculate_from_amount();
            if winning_amt < current_amt {
               winning_tx = tx.clone();
               winning_amt = current_amt;
           }
        }
        if winning_amt == 0.0 {
            return None
        } else {
            return Some(winning_tx.clone());
        }
    }
}


#[derive(Debug)]
pub struct Mempool {
    blocks: RefCell<Vec<Block>>,
    transactions: RefCell<Vec<Transaction>>,
}

impl Mempool {
    pub fn new() -> RefCell<Mempool> {
        return RefCell::new(Mempool{
            blocks: RefCell::new(Vec::new()),
            transactions: RefCell::new(Vec::new())
        });
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.borrow_mut().push(tx);
    }

    pub fn return_transactions(&self) -> Vec<Transaction> {
        return self.transactions.borrow_mut().clone();
    }

    pub fn return_transaction_length(&self) -> u32 {
        return self.transactions.borrow_mut().len() as u32;
    }

    pub fn clear_tx_mempool(&mut self) {
        self.transactions = RefCell::new(Vec::new());
    }
}

#[derive(Debug)]
struct GoldenTicket {
    target: Vec<u8>,
    vote: u8,
    random: Vec<u8>,
    publickey: PublicKey
}

impl GoldenTicket {
    fn calculate_difficulty (&self, previous_block: &Block) -> f32 {
        return match self.vote {
            1 => previous_block.difficulty + 0.01,
            _ => previous_block.difficulty - 0.01
        }
    }

    fn calculate_paysplit (&self, previous_block: &Block) -> f32 {
        return match self.vote {
            1 => previous_block.paysplit + 0.01,
            _ => previous_block.paysplit - 0.01
        }
    }
}

// 0 = normal
// 1 = golden ticket
// 2 = fee transaction
// 3 = rebroadcasting
// 4 = VIP rebroadcast
// 5 = floating coinbase / golden chunk

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub enum TransactionType {
  Base,
  GoldenTicket,
  Fee,
  Rebroadcast,
  VIP,
  GoldenChunk,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Transaction {
    id: u32,
    tx_type: TransactionType,
    timestamp: u128,
    sig: Signature,
    to: Vec<Slip>,
    from: Vec<Slip>,

    #[serde(with = "serde_bytes")]
    pub msg: Vec<u8>,
}

impl Transaction {
    pub fn new(tx_type: TransactionType) -> Transaction {
        return Transaction {
            id: 0,
            timestamp: time_since_unix_epoch(),
            tx_type,
            sig: Signature::from_compact(&[0; 64]).unwrap(),
            to: Vec::new(),
            from: Vec::new(),
            msg: Vec::new()
        };
    }

    fn add_to_slip(&mut self, slip: Slip) {
        self.to.push(slip);
    }

    fn add_from_slip(&mut self, slip: Slip) {
        self.from.push(slip)
    }

    fn calculate_from_amount(&self) -> f32 {
        let mut total_amount: f32 = 0.0;
        for slip in self.from.clone().into_iter() {
           total_amount += slip.amount;
        }
        return total_amount;
    }

    fn calculate_to_amount(&self) -> f32 {
        let mut total_amount: f32 = 0.0;
        for slip in self.to.clone().into_iter() {
           total_amount += slip.amount;
        }
        return total_amount;
    }

    fn return_fees_usable(&self, key: &PublicKey) -> f32 {
        let mut input_fees: f32 = 0.0;
        let mut output_fees: f32 = 0.0;

        for slip in self.from.iter() {
            if &slip.address == key {
                input_fees += slip.amount;
            }
        }

        for slip in self.to.iter() {
            if &slip.address == key {
                output_fees += slip.amount;
            }
        }

        return input_fees - output_fees;
    }


    // duplicate minus id, evaluate
    pub fn return_signature_source(&self) -> Vec<u8> {
        let mut sig_source_bytes: Vec<u8> = Vec::new();
        let timestamp_bytes: [u8; 16] = unsafe { transmute(self.timestamp.to_be()) };

        sig_source_bytes.extend(&timestamp_bytes);

        for slip in self.from.iter() {
            sig_source_bytes.extend(slip.return_index());
        }

        for slip in self.to.iter() {
            sig_source_bytes.extend(slip.return_index());
        }
        return sig_source_bytes;
    }
}

impl Clone for Transaction {
    fn clone(&self) -> Transaction {
        Transaction {
            id: self.id,
            tx_type: self.tx_type,
            timestamp: self.timestamp,
            sig: self.sig,
            to: self.to.clone(),
            from: self.from.clone(),
            msg: self.msg.clone()
        }
    }
}

// finish Hashable for Transaction
impl Hashable for Transaction {
    fn update_context(&self, context: &mut Context) {
        context.update(&self.return_signature_source());
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Slip {
    address: PublicKey,
    amount: f32,
    block_id: u32,
    transaction_id: u32,
    id: u32,

    #[serde(with = "serde_bytes")]
    block_hash: Vec<u8>,
    lc: u8,
}

impl Slip {
    pub fn return_index(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        let address_bytes: Vec<u8> = self.address.serialize().into_iter().cloned().collect();
        let amount_bytes: [u8; 4] = unsafe { transmute(self.amount as u32) };
        let block_id_bytes : [u8; 4] = unsafe { transmute(self.block_id.to_be()) };
        let transaction_id_bytes : [u8; 4] = unsafe { transmute(self.transaction_id.to_be()) };
        let slip_id_bytes : [u8; 4] = unsafe { transmute(self.id.to_be()) };

        //bytes.extend(&address_bytes);
        for elem in address_bytes.iter() { bytes.push(*elem); }
        for elem in amount_bytes.iter() { bytes.push(*elem) }
        for elem in block_id_bytes.iter() { bytes.push(*elem); }
        for elem in transaction_id_bytes.iter() { bytes.push(*elem); }
        for elem in slip_id_bytes.iter() { bytes.push(*elem); }
        for elem in self.block_hash.iter() { bytes.push(*elem); }
        bytes.push(self.lc);

        return bytes;
    }
}

pub struct Wallet {
    publickey: PublicKey,
    privatekey: SecretKey,
    inputs: HashMap<Vec<u8>, Slip>,
    outputs: HashMap<Vec<u8>, Slip>,
    spends: HashMap<Vec<u8>, Slip>
}

impl Wallet {
    pub fn new() -> Wallet {
        let (secret_key, public_key) = generate_keys();
        return Wallet {
            publickey: public_key,
            privatekey: secret_key,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            spends: HashMap::new()
        };
    }

    pub fn return_publickey(&self) -> PublicKey {
        return self.publickey;
    }

    pub fn create_signature(&self, data: &[u8]) -> Signature {
        let mut hasher = Sha256::new();
        hasher.input(data);

        let sign = Secp256k1::signing_only();
        let msg = Message::from_slice(hasher.result().as_slice()).unwrap();
        return sign.sign(&msg, &self.privatekey)
    }

    pub fn return_base58(&self) -> String {
        return self.publickey.serialize().to_base58();
    }

    pub fn add_input(&mut self, input: Slip) {
        self.inputs.insert(input.return_index(), input);
    }

    pub fn add_output(&mut self, output: Slip) {
        self.outputs.insert(output.return_index(), output);
    }

    pub fn process_payment(&mut self, transactions: Vec<Transaction>) {
        for tx in transactions.iter() {
            for slip in tx.from.iter() {
                if slip.address == self.publickey  {
                    self.outputs.insert(slip.return_index(), slip.clone());
                    if self.spends.contains_key(&slip.return_index())  {
                        self.spends.remove(&slip.return_index());
                    }
                }
            }

            for slip in tx.to.iter() {
                if slip.address == self.publickey {
                    self.inputs.insert(slip.return_index(), slip.clone());
                }
            }
        }
    }

    pub fn return_balance(&self) -> f32 {
        let mut balance: f32 = 0.0;
        for (_, slip) in self.inputs.clone() {
            balance += slip.amount;
        }

        return balance;
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Block {
    id: u32,

    #[serde(with = "serde_bytes")]
    previous_hash: Vec<u8>,

    #[serde(with = "serde_bytes")]
    merkle_root: Vec<u8>,

    timestamp: u128,
    creator: PublicKey,
    pub transactions: Vec<Transaction>,
    difficulty: f32,
    paysplit: f32,
    treasury: f32,
    coinbase: f32,
    reclaimed: f32
}

impl Block {
    pub fn new(previous_hash: Vec<u8>, publickey: PublicKey) -> Block {
        return Block {
            id: 1,
            timestamp: time_since_unix_epoch(),
            previous_hash,
            merkle_root: Vec::new(),
            creator: publickey,
            transactions: Vec::new(),
            difficulty: 0.0,
            paysplit: 0.5,
            treasury: 2868100000.0,
            coinbase: 0.0,
            reclaimed: 0.0
        };
    }

    pub fn return_block_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        let id_bytes: [u8; 4] = unsafe { transmute(self.id.to_be()) };
        let timestamp_bytes: [u8; 16] = unsafe { transmute(self.timestamp.to_be()) };
        let address_bytes: Vec<u8> = self.creator.serialize().iter().cloned().collect();

        hasher.input(id_bytes);
        hasher.input(self.previous_hash.as_slice());
        hasher.input(timestamp_bytes);
        hasher.input(address_bytes);

        let block_hash = hasher.result();
        return block_hash.to_vec()
    }

    pub fn return_timestamp(&self) -> u128 {
        return self.timestamp;
    }

    pub fn return_transactions(&self) -> Vec<Transaction> {
        return self.transactions.clone();
    }

    pub fn set_merkle_root(&mut self) {
        self.merkle_root = create_merkle_root(self.transactions.clone());
    }

    fn validate(&self, previous_block: &Block) -> bool {
        // check that the new block timestamp is greater than the old one

        // we need a way to fetch the current blocks prevhash instead of just getting the last
        // block in the block chain
        if previous_block.timestamp >= self.timestamp { return false }

        // ensure no duplicate input slips
        let mut tx_input_hashmap: HashMap<Vec<u8>, u8> = HashMap::new();
        for tx in self.transactions.iter() {
            for slip in tx.from.iter() {
                if !tx_input_hashmap.contains_key(&slip.return_index()) {
                    tx_input_hashmap.insert(slip.return_index(), 0);
                } else {
                    println!("DOUBLE SPEND DETECTED");
                    return false;
                }
            }

            // validate non-rebroadcast tx
           // match tx.tx_type {
           //     TransactionType::Base => { if !tx.validate() { return false; } },
           //     TransactionType::GoldenTicket => { if !tx.validate() { return false } },
           //     _ => {},
           // }
        }

        // validate merkle root
        // TODO:
        //if self.merkle_root == return_merkle_root(self.transactions)
        return true;

        // validate burn fee and fee transaction
        // validate golden ticket
        // validate difficulty
        // validate monetary policy

    }

    pub fn bundle(&mut self, blocks: &RefMut<Vec<Block>>, transactions: Vec<Transaction>, last_tx_id: u32, last_slip_id: u32) {
        match blocks.last() {
           Some(previous_block) => {
               self.bundle_with_previous_block(previous_block);
               self.bundle_transactions(transactions, last_tx_id, last_slip_id);
//               self.calculate_difficulty()
           },
           None => {
               self.bundle_transactions(transactions, last_tx_id, last_slip_id);
           }
        }

    }

    fn bundle_with_previous_block(&mut self, previous_block: &Block) {
         self.id = previous_block.id + 1;
         self.treasury = previous_block.treasury + previous_block.reclaimed;
         self.coinbase = self.treasury / GENESIS_PERIOD as f32; // hard code this
         self.treasury = self.treasury - self.coinbase;
         self.previous_hash = previous_block.return_block_hash();
         self.paysplit = previous_block.paysplit;
         self.difficulty = previous_block.difficulty;
    }

    fn bundle_transactions(&mut self, mut transactions: Vec<Transaction>, last_tx_id: u32, last_slip_id: u32) {
        let mut min_slip_id: u32 = last_slip_id;
        let mut min_tx_id: u32 = last_tx_id;

        for tx in transactions.iter_mut() {
            for j in 0..tx.from.len() {
                tx.from[j].id = last_slip_id;
                tx.from[j].block_id = self.id;
                tx.from[j].transaction_id = min_tx_id;
                min_slip_id = min_slip_id + 1;
            }

            for i in 0..tx.to.len() {
                tx.to[i].id = min_slip_id;
                min_slip_id = min_slip_id + 1;
            }

            tx.id = last_tx_id;
            min_tx_id = min_tx_id + 1;

            //println!("{:?}", tx);
            self.transactions.push(tx.clone());
        }
    }

//    fn calculate_difficulty(&mut self) {
//        for tx in self.transactions.iter() {
//            if tx.tx_type == TransactionType::GoldenTicket {
//                match tx.ty_type {
//                    TransactionType::GoldenTicket => {
//                        self.difficulty = tx.calculate_difficulty(previous_block);
//                        self.paysplit = tx.calculate_paysplit(previous_block);
//                    }
//                    _ => {}
//                }
//            }
//        }
//    }
//
    fn return_available_fees(&self, key: &PublicKey) -> f32 {
        let mut total_fees: f32 = 0.0;

        for tx in self.transactions.iter() {
            total_fees += tx.return_fees_usable(key);
        }

        return total_fees;
    }

    pub fn return_slip_len(&self) -> u32 {
        let mut slip_number: u32 = 0;
        for tx in self.transactions.iter() {
            slip_number += tx.to.len() as u32;
            slip_number += tx.from.len()as u32;
        }
        return slip_number
    }

    pub fn return_tx_len(&self) -> u32{
        return self.transactions.len() as u32;
    }
}


#[derive(Debug)]
pub struct Blockchain {
    genesis_ts: u128,
    last_block_id: u32,
    last_tx_id: u32,
    last_slip_id: u32,
    pub blocks: RefCell<Vec<Block>>,

}

impl Blockchain {
    pub fn new () -> Blockchain {
        return Blockchain {
            genesis_ts: time_since_unix_epoch(),
            blocks: RefCell::new(Vec::new()),
            last_block_id: 0,
            last_tx_id: 1,
            last_slip_id: 1
        };
    }

    pub fn return_previous_hash(&self) -> Vec<u8> {
        return self.blocks.borrow_mut().last().unwrap().return_block_hash();
    }

    pub fn return_last_block_id(&self) -> u32 {
        return self.last_block_id;
    }

    pub fn return_last_tx_id(&self) -> u32 {
        return self.last_tx_id;
    }

    pub fn return_last_slip_id(&self) -> u32 {
        return self.last_slip_id;
    }

    pub fn return_blocks_length(&self) -> usize {
        return self.blocks.borrow_mut().len();
    }

    pub fn increment_block_id(&mut self) {
        self.last_block_id = self.last_block_id + 1;
    }

    pub fn update_tx_id(&mut self, tx_id: u32) {
        self.last_tx_id = tx_id;
    }

    pub fn update_slip_id(&mut self, slip_id: u32) {
        self.last_slip_id = slip_id;
    }
}

#[derive(Debug)]
pub struct BurnFee {
    fee: f32,
    heartbeat: u32,
    last_block_timestamp: u128,
    last_block_delta: u128
}

impl BurnFee {
    pub fn new() -> BurnFee {
        return BurnFee {
            fee: 10.0,
            heartbeat: 10,
            last_block_timestamp: time_since_unix_epoch(),
            last_block_delta: 0
        };
    }

    pub fn calculate(&self, mut elapsed_time: u128) -> f32 {
        //let mut elapsed_time = time_since_unix_epoch()  - self.last_block_timestamp;

        // return 0.0 if it's been twice as long as 10s
        if (elapsed_time / 1000) > (self.heartbeat as u128 * 2) { return 0.0; }

        if elapsed_time == 0 { elapsed_time = 1; }

        let elapsed_time_float = elapsed_time as f32;
        return self.fee / (elapsed_time_float / 1000.0);
    }

    pub fn return_current_burnfee(&self) -> f32 {
        return self.calculate(time_since_unix_epoch() - self.last_block_timestamp);
    }

    fn return_previous_burnfee(&self) -> f32 {
        return self.calculate(self.last_block_delta);
    }

    pub fn set_timestamp(&mut self, new_block_timestamp: u128) {
        self.last_block_timestamp = new_block_timestamp;
    }

    pub fn set_last_block_delta(&mut self, new_block_timestamp: u128) {
        self.last_block_delta = new_block_timestamp - self.last_block_timestamp;
    }

    pub fn adjust(&mut self, current_block_timestamp: u128) {
        let numerator = (self.heartbeat as f32 * 10000000.0).sqrt();
        let denominator = current_block_timestamp as u32 - self.last_block_timestamp as u32 + 1;

        self.fee = self.fee * (numerator / denominator as f32);
    }
}
