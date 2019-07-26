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