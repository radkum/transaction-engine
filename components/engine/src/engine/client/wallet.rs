use std::fmt::{Display, Formatter};

use rust_decimal::prelude::*;

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
        //write!(f, "{:.4},{:.4},{:.4},{:.4}", self.available, self.held, self.total, self.locked)

        //we can use rust_decimal crate to round and display it without trailing zeros
        write!(
            f,
            "{},{},{},{}",
            Self::round(self.available),
            Self::round(self.held),
            Self::round(self.total),
            self.locked
        )
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
        if Self::is_equal_f32(self.available, amount) {
            self.available = 0.0;
            self.total = 0.0;
        } else if self.available > amount {
            self.available -= amount;
            self.total -= amount;
        } else {
            log::warn!(
                "There is no enough funds to withdraw. Available funds: {}, requested: {amount}",
                self.available
            );
        }
    }

    pub(super) fn dispute(&mut self, amount: f32) -> bool {
        //println!("dispute {}, {}", self.available, amount);
        if Self::is_equal_f32(self.available, amount) {
            self.available = 0.0;
            self.held = amount;
            true
        } else if self.available > amount {
            self.available -= amount;
            self.held += amount;
            true
        } else {
            log::warn!(
                "There is no enough funds to dispute. Available funds: {}, requested: {amount}",
                self.available
            );
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

    // helper functions
    fn is_equal_f32(first: f32, second: f32) -> bool {
        // we need precision of 4'th place so epsilon is 0.0001 / 2
        const E: f32 = 0.10005;

        // simplest case: two floats are identical
        if first == second {
            return true;
        }

        // calculation on floats cause not ideal results. We must take this into account
        if first > second && (first - second) < E {
            true
        } else if second > first && (second - first) < E {
            true
        } else {
            false
        }
    }

    fn round(first: f32) -> Decimal {
        // this can't be infinite or NaN because we have only addition and subtraction
        let dec = Decimal::from_f32(first).unwrap_or_default();
        dec.round_dp(4).into()
    }
}
