use crate::error::{
	ConvertPublicKeyToAddressError, LoadOrCreateUserError, NewUserError,
	UserSignError, ValidateUserSignatureError,
};

/// The structure of the user, which stores his private key and address.
///
/// Use `Self::load_or_create` to get the object of an existing or new user.
#[derive(Clone, Debug)]
pub struct User {
	address: String,
	private_key: k256::ecdsa::SigningKey,
}

impl User {
	common::accessor!(& address -> &str);

	/// Loads or creates a new user depending on whether
	/// `consts::PRIVATE_KEY_PATH` exists.
	#[tracing::instrument(ret)]
	pub fn load_or_create() -> Result<Self, LoadOrCreateUserError> {
		if crate::consts::PRIVATE_KEY_PATH.exists() {
			tracing::info!("Loading an existing user...");
			// Read bytes and convert them to a private key
			let bytes =
				std::fs::read(crate::consts::PRIVATE_KEY_PATH.as_path())
					.map_err(LoadOrCreateUserError::Read)?;
			let key = k256::ecdsa::SigningKey::from_bytes(&bytes)?;
			return Ok(Self::new(key)?);
		}

		tracing::info!("Creating a new user...");
		// Generating a private key and writing it to a file
		let key = k256::ecdsa::SigningKey::random(rand::rngs::OsRng);
		std::fs::write(
			crate::consts::PRIVATE_KEY_PATH.as_path(),
			key.to_bytes(),
		)
		.map_err(LoadOrCreateUserError::Write)?;
		Ok(Self::new(key)?)
	}

	#[tracing::instrument(ret)]
	pub(crate) fn new(
		private_key: k256::ecdsa::SigningKey,
	) -> Result<Self, NewUserError> {
		let public_key = private_key.verifying_key();
		let address = Self::convert_public_key_to_address(public_key)?;
		Ok(Self { address, private_key })
	}

	/// A shorthand for validating the signature, having only the signature,
	/// the data, and the address of the person who signed the data.
	///
	/// # Params
	///
	/// `signature`: Base58-formatted string obtained with `self.sign`.
	#[tracing::instrument(level = tracing::Level::DEBUG, ret)]
	pub(crate) fn validate_signature(
		signature: &str,
		data: &str,
		address: &str,
	) -> Result<(), ValidateUserSignatureError> {
		use {
			base58::FromBase58 as _,
			k256::ecdsa::signature::{Signature as _, Verifier as _},
		};

		// Recovering a public key from a `Signature`
		let signature_bytes = signature
			.from_base58()
			.map_err(ValidateUserSignatureError::FromBase58)?;
		let signature =
			k256::ecdsa::recoverable::Signature::from_bytes(&signature_bytes)
				.map_err(ValidateUserSignatureError::Parse)?;
		let key = signature
			.recover_verify_key(data.as_bytes())
			.map_err(ValidateUserSignatureError::RecoverKey)?;
		// Compare adresses
		if Self::convert_public_key_to_address(key)? != address {
			return Err(ValidateUserSignatureError::AddressesNotEquals);
		}
		// Verify
		key.verify(data.as_bytes(), &signature)
			.map_err(ValidateUserSignatureError::Verify)
	}

	/// Signs the data with the private key and returns the signature in Base58
	/// format.
	#[tracing::instrument(level = tracing::Level::DEBUG, ret)]
	pub(crate) fn sign(&self, data: &str) -> Result<String, UserSignError> {
		use {base58::ToBase58 as _, k256::ecdsa::signature::Signer as _};

		let signature: k256::ecdsa::recoverable::Signature =
			self.private_key.try_sign(data.as_bytes())?;
		Ok(signature.as_ref().to_base58())
	}

	/// Converts a public key into an address, just like bitcoin does.
	#[tracing::instrument(level = tracing::Level::DEBUG, ret)]
	fn convert_public_key_to_address(
		public_key: k256::ecdsa::VerifyingKey,
	) -> Result<String, ConvertPublicKeyToAddressError> {
		use {base58::ToBase58 as _, sha2::Digest as _};

		let bytes = [&[4], public_key.to_bytes().as_slice()].concat();
		// Hashing
		let sha256_hash = sha2::Sha256::digest(&bytes);
		let ripemd160_hash = ripemd160::Ripemd160::digest(&sha256_hash);
		// Add network byte and get checksum
		let prepend_network_byte = [vec![0], ripemd160_hash.to_vec()].concat();
		let checksum = crate::helpers::get_checksum(&prepend_network_byte);
		// Encode it to hex and add the checksum
		let mut hex = hex::encode(prepend_network_byte);
		hex.push_str(&checksum);
		// hex to Base-58
		Ok(hex::decode(hex)?.to_base58())
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_convert_public_key_to_address() {
		use base58::FromBase58 as _;

		let user = crate::test_helpers::create_test_user();
		let len = user.address.len();

		assert!((26..35).contains(&len));
		assert!(user.address.from_base58().is_ok());
		Ok(())
	}

	#[test]
	fn test_sign() {
		const DATA: &str = "DATA";
		let user = crate::test_helpers::create_test_user();
		let signature = user.sign(DATA).unwrap();
		super::User::validate_signature(&signature, DATA, &user.address)
			.unwrap();
		Ok(())
	}
}
