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
