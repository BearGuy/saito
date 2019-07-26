
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