
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