#[cfg_attr(test, mockall_double::double)]
use self::wallet::WalletHelpers;

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

    #[cfg(test)]
    pub fn mock() -> Self {
        let wallet_helpers = WalletHelpers::default();
        Self { wallet_helpers }
    }
    #[cfg(test)]
    pub fn mut_wallet_helpers(&mut self) -> &mut WalletHelpers {
        &mut self.wallet_helpers
    }

    #[cfg(test)]
    pub fn with_mock_wallet_helpers(wallet_helpers: WalletHelpers) -> Self {
        Self { wallet_helpers }
    }
    #[cfg(test)]
    pub fn mock_wallet_helpers() -> WalletHelpers {
        WalletHelpers::default()
    }
}
