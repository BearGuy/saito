use saito::{Mempool, Blockchain, BurnFee, Wallet, Miner, Block, create_merkle_root};

use std::cell::{RefCell, RefMut};
use std::time::{SystemTime, UNIX_EPOCH};

use std::{thread, time};

fn main() {
    println!("Running Saito");

    let mut mempool = Mempool::new();
    let mut blockchain = Blockchain::new();
    let mut burnfee = BurnFee::new();
    let mut wallet = Wallet::new();
    let mut miner = Miner::new();

    
    // Initialize our blockchain state and start mining
    blockchain.initialize(&mut wallet);
    miner.initialize(&mempool, &blockchain.blocks.borrow_mut(), &wallet, &burnfee);

    let public_key_base_58 = wallet.return_base58();
    println!("YOUR PUBLICKEY: {}", public_key_base_58);

    loop {
        let num_tx_in_mempool = mempool.borrow_mut().return_transaction_length();

        if (burnfee.return_current_burnfee() <= 0.0 && num_tx_in_mempool > 0) || blockchain.return_last_block_id() == 0 {
            miner.stop_mining();

            let mut previous_hash: Vec<u8> = Vec::new();

            if blockchain.return_blocks_length() > 0 {
                previous_hash = blockchain.return_previous_hash();
            }

            let mut block = Block::new(previous_hash, wallet.return_publickey());

            // transfer all of the transactions of the mempool into our block
            block.bundle(
                &blockchain.blocks.borrow_mut(),
                mempool.borrow_mut().return_transactions(),
                blockchain.return_last_tx_id(),
                blockchain.return_last_slip_id(),
            );

            let last_tx_id: u32 = blockchain.return_last_tx_id() + block.return_tx_len();
            let last_slip_id: u32 = blockchain.return_last_slip_id() + block.return_slip_len();

            // clear the mempool afterwards
            mempool.borrow_mut().clear_tx_mempool();

            // create merkle root for block once transactions are collected
            // block.merkle_root = create_merkle_root(block.transactions.clone());
            block.set_merkle_root();

            let current_block_timestamp = block.return_timestamp();

            burnfee.adjust(current_block_timestamp);
            burnfee.set_timestamp(current_block_timestamp);
            burnfee.set_last_block_delta(current_block_timestamp);

            println!("{:?}", block);

            if !blockchain.validate_block(&block) { 
                println!("BLOCK INVALID, SHUTTING DOWN");
                return;
            }

            // update our slips
            block.update_slips();

            // process them into our wallet afterwards 
            wallet.process_payment(block.transactions.borrow());
            println!("CURRENT BALANCE: {}", wallet.return_balance());

            //block.save();
            blockchain.add_block(block);

            // Possibly add these to block?
            blockchain.increment_block_id();
            blockchain.update_tx_id(last_tx_id);
            blockchain.update_slip_id(last_slip_id);

            println!("Block has been added to the chain!");


            // need to borrow block and implement it
            miner.start_mining(&mempool, &blockchain.blocks.borrow_mut().last().unwrap(), &wallet, &burnfee);

            println!("STARTING MINING ON NEW BLOCK");

            // reset the burnfee by changing the timestamp
        } else {
            let one_second = time::Duration::from_millis(1000);
            thread::sleep(one_second);
            println!("FEE -- {:.8}", burnfee.return_current_burnfee());
        }
    }

}



