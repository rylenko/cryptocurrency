/// Information that is sent to a node when a new block is added.
#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct BlockAddInfo<'a> {
	block: std::borrow::Cow<'a, blockchain::block::Block<'a>>,
	blockchain_len: usize,
}

impl<'a> BlockAddInfo<'a> {
	common::accessor!(& block -> &blockchain::block::Block);

	common::accessor!(copy blockchain_len -> usize);

	#[inline]
	#[must_use]
	pub fn new(
		block: &'a blockchain::block::Block<'a>,
		blockchain_len: usize,
	) -> Self {
		Self { block: std::borrow::Cow::Borrowed(block), blockchain_len }
	}
}
