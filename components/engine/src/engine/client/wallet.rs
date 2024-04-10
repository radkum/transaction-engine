use std::fmt::{Display, Formatter};

pub(crate) struct Wallet {
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
}

impl Default for Wallet {
    fn default() -> Self {
        Self { available: 0.0, held: 0.0, total: 0.0, locked: false }
    }
}
impl Display for Wallet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{},{}", self.available, self.held, self.total, self.locked)
    }
}

impl Wallet {
    pub(super) fn locked(&mut self) -> bool {
        self.locked
    }

    pub(super) fn deposit(&mut self, amount: f32) {
        self.available += amount;
        self.total += amount;
    }

    pub(super) fn withdrawal(&mut self, amount: f32) {
        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
        } else {
            log::warn!("There is no enough funds to withdraw. Available funds: {amount}");
        }
    }

    pub(super) fn dispute(&mut self, amount: f32) -> bool {
        if self.available >= amount {
            self.available -= amount;
            self.held += amount;
            true
        } else {
            log::warn!("There is no enough funds to dipsute. Available funds: {amount}");
            false
        }
    }

    pub(super) fn resolve(&mut self, amount: f32) {
        self.available += amount;
        self.held -= amount;
    }

    pub(super) fn chargeback(&mut self, amount: f32) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
    }
}
