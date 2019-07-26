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