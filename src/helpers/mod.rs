use crate::import_double;

import_double!(self::wallet::WalletHelpers);

mod wallet;

pub struct Helpers {
    wallet_helpers: WalletHelpers,
}

impl Helpers {
    pub fn new() -> Self {
        let wallet_helpers = WalletHelpers::new();
        Self { wallet_helpers }
    }
    pub fn wallet_helpers(&self) -> &WalletHelpers {
        &self.wallet_helpers
    }
}

#[cfg(test)]
impl Helpers {
    pub fn mock() -> Self {
        let wallet_helpers = WalletHelpers::default();
        Self { wallet_helpers }
    }
    pub fn mut_wallet_helpers(&mut self) -> &mut WalletHelpers {
        &mut self.wallet_helpers
    }
}
