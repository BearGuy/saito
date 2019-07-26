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