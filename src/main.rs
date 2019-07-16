extern crate sha2;
extern crate rand;
extern crate base58;
extern crate secp256k1;
extern crate merkle;

use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};

use std::cell::{RefCell, RefMut};

use std::mem::transmute;

use std::collections::HashMap;

use sha2::Sha256;
use digest::Digest;

use ring::digest::{SHA256, Context};

use secp256k1::{Secp256k1, Message, Signature};
use secp256k1::{SecretKey, PublicKey};

use merkle::{MerkleTree, Hashable};

use rand::{Rng,thread_rng};

use base58::{ToBase58};

static GENESIS_PERIOD: i32 = 21600;

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
fn create_merkle_root(transactions: Vec<Transaction>) -> Vec<u8> {
    let merkle = MerkleTree::from_vec(&SHA256, transactions);
    let merkle_root: Vec<u8> = merkle.root_hash().clone();
    return merkle_root;
}


#[derive(Debug)]
struct Miner {
    is_mining: bool,
    can_i_mine: bool,
}

impl Miner {
    //fn start_mining(&mut self, mut mempool: Mempool, wallet: &Wallet, prev_blk: &Block) {
    fn start_mining(&mut self, mempool: &RefCell<Mempool>, wallet: &Wallet, prev_blk: &Vec<u8>) {
        self.can_i_mine = true; 
        self.is_mining = true;

        let golden_ticket = GoldenTicket {
            target: String::from("TARGET"),
            vote: String::from("VOTE"),
            random: String::from("RANDOM"),
            publickey: wallet.publickey,
        };
        
        while self.can_i_mine {
            self.attempt_solution(mempool, &golden_ticket, prev_blk);
        };
    }

    fn stop_mining(&mut self) {
        self.can_i_mine = false;
    }

    fn attempt_solution(&mut self, mempool: &RefCell<Mempool>, golden_ticket: &GoldenTicket, prev_blk: &Vec<u8>) {
        //if prev_blk.id == 1 { return; } 
        
        let mut rng = thread_rng(); 
        let random_number = rng.gen::<u32>();

        let random_number_bytes: [u8; 4] = unsafe { transmute(random_number.to_be()) };
        
        // hash our solution
        let mut hasher = Sha256::new();
        let publickey_vec: Vec<u8> = golden_ticket.publickey.serialize().iter().cloned().collect();
        hasher.input(publickey_vec);
        hasher.input(random_number_bytes);

        //let random_solution = hasher.result().as_slice();

        if self.is_valid_solution(hasher.result().as_slice(), prev_blk) {
            // Stop mining
            println!("WE HAVE FOUND A SOLUTION");
            self.can_i_mine = false;

            // Find winning node
            // let winning_tx_id = self.find_winner();

            
            // Calculate amount won
            // static amount
            let node_amount = 1000.0;
            let miner_amount = 1000.0;

            let mut golden_tx = Transaction {
                id: 0,
                timestamp: time_since_unix_epoch(), 
                tx_type: TransactionType::GoldenTicket,
                to: Vec::new(),
                from: Vec::new(),
            };
            
            golden_tx.add_to_slip(Slip {
                address: golden_ticket.publickey,
                amount: node_amount,
                block_id: 0,
                transaction_id: 0,
                id: 0,
                block_hash: [0; 32],
                lc: 1
            });

            golden_tx.add_to_slip(Slip {
                address: golden_ticket.publickey,
                amount: miner_amount,
                block_id: 0,
                transaction_id: 0,
                id: 0,
                block_hash: [0; 32],
                lc: 1 
            });
            
            golden_tx.add_from_slip(Slip {
                address: golden_ticket.publickey,
                amount: 0.0,
                block_id: 0,
                transaction_id: 0,
                id: 0,
                block_hash: [0; 32],
                lc: 1 
            });


            mempool.borrow_mut().add_transaction(golden_tx);
        }
    }

    fn is_valid_solution(&self, random_solution: &[u8], prev_blk: &Vec<u8>) -> bool {
        // static difficulty until this is implemented on the block object 
        let difficulty = 2;

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

    fn find_winner(&self) -> u32 {
       // let max_hash = 0xFFFFFFFF;
       return 0
    }
}


#[derive(Debug)]
struct Mempool {
    blocks: RefCell<Vec<Block>>,
    transactions: RefCell<Vec<Transaction>>,
}

impl Mempool {
    fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.borrow_mut().push(tx);
    }
}

#[derive(Debug)]
struct GoldenTicket {
    target: String,
    vote: String,
    random: String,
    publickey: PublicKey
}

// 0 = normal
// 1 = golden ticket
// 2 = fee transaction
// 3 = rebroadcasting
// 4 = VIP rebroadcast
// 5 = floating coinbase / golden chunk

#[derive(Debug, Copy, Clone)]
enum TransactionType {
  Base, 
  GoldenTicket,
  Fee,
  Rebroadcast,
  VIP,
  GoldenChunk,
}

#[derive(Debug)]
struct Transaction {
    id: u32, 
    tx_type: TransactionType,
    timestamp: u128,
    to: Vec<Slip>,
    from: Vec<Slip>
}

impl Transaction {
    fn add_to_slip(&mut self, slip: Slip) {
        self.to.push(slip); 
    }

    fn add_from_slip(&mut self, slip: Slip) {
        self.from.push(slip)
    }

    pub fn return_index(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        let id_bytes: [u8; 4] = unsafe { transmute(self.id.to_be()) };
        let timestamp_bytes: [u8; 16] = unsafe { transmute(self.timestamp.to_be()) };

        bytes.extend(&id_bytes);
        bytes.extend(&timestamp_bytes);

        for elem in self.to.iter() {
            bytes.extend(&elem.return_index());
        }
        
        for elem in self.from.iter() {
            bytes.extend(&elem.return_index());
        }
        return bytes
    }
}

impl Clone for Transaction {
    fn clone(&self) -> Transaction {
        Transaction {
            id: self.id,
            tx_type: self.tx_type,
            timestamp: self.timestamp,
            to: self.to.clone(),
            from: self.from.clone()
        }
    }
}

// finish Hashable for Transaction
impl Hashable for Transaction {
    fn update_context(&self, context: &mut Context) {
        context.update(&self.return_index());
    }
}

#[derive(Copy, Clone, Debug)]
struct Slip {
    address: PublicKey,
    amount: f32,
    block_id: u32,
    transaction_id: u32,
    id: u32,
    block_hash: [u8; 32],
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

struct Wallet {
    publickey: PublicKey,
    privatekey: SecretKey,
    inputs: HashMap<Vec<u8>, Slip>,
    outputs: HashMap<Vec<u8>, Slip>,
    spends: HashMap<Vec<u8>, Slip>
}

impl Wallet {
    fn create_signature(&self, msg: &[u8]) -> Signature {
        let sign = Secp256k1::signing_only();
        let msg = Message::from_slice(&msg).unwrap();
        return sign.sign(&msg, &self.privatekey)
    }

    fn return_base58(&self) -> String {
        return self.publickey.serialize().to_base58();
    }

    fn add_input(&mut self, input: Slip) {
        self.inputs.insert(input.return_index(), input);
    }

    fn add_output(&mut self, output: Slip) {
        self.outputs.insert(output.return_index(), output);
    }

    fn process_payment(&mut self, transactions: Vec<Transaction>) {
        for tx in transactions.iter() {
            for slip in tx.from.iter() {
                if slip.address == self.publickey  {
                    self.outputs.insert(slip.return_index(), *slip);
                    if self.spends.contains_key(&slip.return_index())  {
                        self.spends.remove(&slip.return_index());
                    }
                }
            }

            for slip in tx.to.iter() {
                if slip.address == self.publickey {
                    self.inputs.insert(slip.return_index(), *slip);
                }
            }
        }
    }

    fn return_balance(&self) -> f32 {
        let mut balance: f32 = 0.0; 
        for (_, slip) in self.inputs.clone() {
            balance += slip.amount;
        }

        return balance;
    }
}

#[derive(Debug)]
struct Block {
    id: u32,
    previous_hash: Vec<u8>, 
    merkle_root: Vec<u8>,
    timestamp: u128,
    creator: PublicKey,
    transactions: Vec<Transaction>,
    difficulty: f32,
    paysplit: f32,
    treasury: f32, 
    coinbase: f32,
    reclaimed: f32
}

impl Block {
    fn return_block_hash(&self) -> Vec<u8> {
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

    pub fn bundle(&mut self, blocks: &RefMut<Vec<Block>>, transactions: Vec<Transaction>, last_tx_id: u32, last_slip_id: u32) {
        match blocks.last() {
           Some(previous_block) => {
               self.bundle_with_previous_block(previous_block);
               self.bundle_transactions(transactions, last_tx_id, last_slip_id);
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

    fn return_slip_len(&self) -> u32 {
        let mut slip_number: u32 = 0;
        for tx in self.transactions.iter() {
            slip_number += tx.to.len() as u32;
            slip_number += tx.from.len()as u32;
        }
        return slip_number
    }

    fn return_tx_len(&self) -> u32{
        return self.transactions.len() as u32;
    }
}


#[derive(Debug)]
struct Blockchain {
    genesis_ts: u128,
    last_block_id: u32,
    last_tx_id: u32,
    last_slip_id: u32,
    blocks: RefCell<Vec<Block>>,
    
}
  
impl Blockchain {
    fn return_previous_hash(&self) -> Vec<u8> {
       return self.blocks.borrow_mut().last().unwrap().return_block_hash();
    }

    fn increment_block_id(&mut self) {
        self.last_block_id = self.last_block_id + 1;
    }

    fn update_tx_id(&mut self, tx_id: u32) {
        self.last_tx_id = tx_id;
    }

    fn update_slip_id(&mut self, slip_id: u32) {
        self.last_slip_id = slip_id;
    }
}

//impl Mempool {
//    fn bundle(&self) {
//        println!("BUNDLE BLOCK!");
//    }
//}

#[derive(Debug)]
struct BurnFee {
    fee: f32,
    heartbeat: u32,
    last_block_timestamp: u128,
}

impl BurnFee {
    fn calculate(&self) -> f32 {
        let mut elapsed_time = time_since_unix_epoch()  - self.last_block_timestamp;

        // return 0.0 if it's been twice as long as 10s 
        if (elapsed_time / 1000) > (self.heartbeat as u128 * 2) { return 0.0; }

        if elapsed_time == 0 { elapsed_time = 1; }

        let elapsed_time_float = elapsed_time as f32;
        return self.fee / (elapsed_time_float / 1000.0);
    }

    fn set_timestamp(&mut self, new_block_timestamp: u128) {
        self.last_block_timestamp = new_block_timestamp;
    }

    fn adjust(&mut self, current_block_timestamp: u128) {
        let numerator = (self.heartbeat as f32 * 10000000.0).sqrt();
        let denominator = current_block_timestamp as u32 - self.last_block_timestamp as u32 + 1;

        self.fee = self.fee * (numerator / denominator as f32);
    }
}

fn main() {
    println!("Running Saito");

    // Create Mempool
    let mut mempool = RefCell::new(Mempool{
        blocks: RefCell::new(Vec::new()), 
        transactions: RefCell::new(Vec::new())
    }); 

    let mut blockchain = Blockchain { 
        genesis_ts: time_since_unix_epoch(),
        blocks: RefCell::new(Vec::new()),
        last_block_id: 0,
        last_tx_id: 1,
        last_slip_id: 1
    };
    
    let mut burnfee = BurnFee { 
        fee: 10.0, 
        heartbeat: 10,
        last_block_timestamp: time_since_unix_epoch(),
    };
    
    let (secret_key, public_key) = generate_keys(); 
    let mut wallet = Wallet {
        publickey: public_key,
        privatekey: secret_key,
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        spends: HashMap::new()
    };

    let mut miner = Miner {
        is_mining: false,
        can_i_mine: false
    };
    
    let public_key_base_58 = wallet.return_base58();
    println!("YOUR PUBLICKEY: {}", public_key_base_58);

    loop {
        let num_tx_in_mempool = mempool.borrow_mut().transactions.borrow_mut().len(); 
        if (burnfee.calculate() <= 0.0 && num_tx_in_mempool > 0) || blockchain.last_block_id == 0 {
            miner.stop_mining(); 
            
            let mut previous_hash: Vec<u8> = Vec::new();
           
            println!("{}", blockchain.blocks.borrow_mut().len());
            
            if blockchain.blocks.borrow_mut().len() > 0 {
                previous_hash = blockchain.return_previous_hash();
            }

            let mut block = Block { 
                id: 1,
                timestamp: time_since_unix_epoch(),
                previous_hash,
                merkle_root: Vec::new(),
                creator: wallet.publickey,
                transactions: Vec::new(),
                difficulty: 0.0,
                paysplit: 0.5,
                treasury: 2868100000.0,  
                coinbase: 0.0,
                reclaimed: 0.0
            };

            // transfer all of the transactions of the mempool into our block
            // bundle block
            // block.transactions = mempool.borrow_mut().transactions.borrow_mut().clone();
            block.bundle(
                &blockchain.blocks.borrow_mut(), 
                mempool.borrow_mut().transactions.borrow_mut().clone(),
                blockchain.last_tx_id,
                blockchain.last_slip_id, 
            );

            let last_tx_id: u32 = blockchain.last_tx_id + block.return_tx_len();
            let last_slip_id: u32 = blockchain.last_slip_id + block.return_slip_len();

            // clear the mempool afterwards
            mempool.borrow_mut().transactions = RefCell::new(Vec::new());

            // create merkle root for block once transactions are collected
            block.merkle_root = create_merkle_root(block.transactions.clone());

            let current_block_timestamp = block.timestamp;
            
            burnfee.adjust(current_block_timestamp);
            burnfee.set_timestamp(current_block_timestamp);
            
            println!("{:?}", block);

            wallet.process_payment(block.transactions.clone());
            println!("CURRENT BALANCE: {}", wallet.return_balance());

            blockchain.blocks.borrow_mut().push(block);
 
            // Possibly add these to block? 
            blockchain.increment_block_id();
            blockchain.update_tx_id(last_tx_id);
            blockchain.update_slip_id(last_slip_id);

            println!("Block has been added to the chain!");


            // need to borrow block and implement it
            miner.start_mining(&mempool, &wallet, &blockchain.return_previous_hash());

            println!("STARTING MINING ON NEW BLOCK");

            // reset the burnfee by changing the timestamp
        } else {
            let one_second = time::Duration::from_millis(1000); 
            thread::sleep(one_second);
            println!("FEE -- {:.8}", burnfee.calculate());
        }
    }

}


