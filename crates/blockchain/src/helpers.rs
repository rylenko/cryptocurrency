pub type BalanceState = std::collections::BTreeMap<String, u64>;

/// Gets the checksum of the `data`: Hashes SHA-256 twice, then gets the hex
/// and returns the first 8 characters.
#[must_use]
pub(crate) fn get_checksum(data: &[u8]) -> String {
	use sha2::Digest as _;
	let hash = sha2::Sha256::digest(&sha2::Sha256::digest(data));
	let hex = hex::encode(hash);
	hex[..8].to_owned()
}

/// Generates a random bytes with and converts them into a string.
#[must_use]
pub(crate) fn generate_random_string() -> String {
	use {base58::ToBase58 as _, rand::RngCore as _};
	let mut random_bytes = [0u8; 32];
	rand::thread_rng().fill_bytes(&mut random_bytes);
	random_bytes.to_base58()
}

/// Returns Unix timestamp.
#[must_use]
pub(crate) fn get_timestamp() -> f64 {
	std::time::SystemTime::now()
		.duration_since(std::time::SystemTime::UNIX_EPOCH)
		.expect("`std::time::SystemTime` before the Unix epoch!")
		.as_secs_f64()
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_get_checksum() {
		assert_eq!(super::get_checksum(b"1"), "9c2e4d8f");
	}

	#[test]
	fn test_generate_random_string() {
		let mut results = std::collectionsHashSet::new();
		for _ in 0..50 {
			results.insert(super::generate_random_string());
		}
		assert_eq!(results.len(), 50);
	}
}
