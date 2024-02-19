#[must_use]
pub fn create_test_user() -> User {
	crate::user::User::new(k256::ecdsa::SigningKey::random(rand::rngs::OsRng))
		.unwrap()
}

#[must_use]
pub fn create_test_block<'a>() -> (crate::user::User, crate::block::Block<'a>)
{
	let user = create_test_user();
	let block = crate::block::Block::new(
		user.address().to_owned(),
		None::<&str>,
		crate::block::Transactions::new(),
		crate::helpers::BalanceState::new(),
	);
	(user, block)
}
