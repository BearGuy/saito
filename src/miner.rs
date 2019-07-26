

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