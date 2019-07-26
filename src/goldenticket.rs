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