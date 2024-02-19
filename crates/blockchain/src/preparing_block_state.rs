/// This structure stores the `self.transitions` and `self.balance_state`,
/// which will go into the next block.
#[derive(Clone)]
pub struct PreparingBlockState<'a> {
	pub(crate) transactions: crate::block::Transactions<'a>,
	pub(crate) balance_state: crate::helpers::BalanceState,
}

impl PreparingBlockState<'_> {
	common::accessor!(& transactions -> &crate::block::Transactions);

	pub(crate) fn new() -> Self {
		Self {
			transactions: crate::block::Transactions::new(),
			balance_state: crate::helpers::BalanceState::new(),
		}
	}

	pub fn clear(&mut self) {
		self.transactions.clear();
		self.balance_state.clear();
	}

	#[must_use]
	pub fn filled(&self) -> bool {
		self.transactions.len() >=
			super::consts::USER_TRANSACTIONS_PER_BLOCK as usize
	}
}
