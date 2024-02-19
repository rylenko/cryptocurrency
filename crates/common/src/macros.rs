/// Same as [`println!`] but on behalf of a `node`.
#[macro_export]
macro_rules! nprintln {
	($node:expr, $string:expr) => {
		println!("[{}]: {}", $node, $string);
	};
	($node:expr, $string:expr, $($arg:expr)*) => {
		println!("[{}]: {}", $node, format_args!($string, $($arg)*));
	};
}

/// Works in the same way as [`vec!`], but is used to create a
/// [`std::collections::HashSet`].
#[macro_export]
macro_rules! set {
	($($x:expr),*) => {
		{
			let mut set = std::collections::HashSet::new();
			$( set.insert($x); )*
			set
		}
	};
}

/// Creates a accessor function for struct field.
/// Args format: `operation field -> return_type`
///
/// # Examples
///
/// ```rust
/// # use common::accessor;
/// struct S<T> {
///     a: i32,
///     b: String,
///     c: Option<T>,
///     d: Option<String>,
/// }
/// impl<T> S<T> {
///     accessor!(copy a -> i32);
///     accessor!(& b -> &str);
///     accessor!(as_ref c -> Option<&T>);
///     accessor!(as_deref d -> Option<&str>);
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! accessor (
	(copy $field:ident -> $return_type:ty) => {
		#[inline]
		#[must_use]
		pub fn $field(&self) -> $return_type { self.$field }
	};
	(& $field:ident -> $return_type:ty) => {
		#[inline]
		#[must_use]
		pub fn $field(&self) -> $return_type { &self.$field }
	};
	(&mut $method:ident($field:ident) -> $return_type:ty) => {
		#[inline]
		#[must_use]
		pub fn $method(&mut self) -> $return_type { &mut self.$field }
	};
	(as_ref $field:ident -> $return_type:ty) => {
		#[inline]
		#[must_use]
		pub fn $field(&self) -> $return_type { self.$field.as_ref() }
	};
	(as_deref $field:ident -> $return_type:ty) => {
		#[inline]
		#[must_use]
		pub fn $field(&self) -> $return_type { self.$field.as_deref() }
	};
);

/// [`Receive`](common::package::Package::receive) a
/// `common::package::Package`, or `common::nprintln` error and `continue`.
#[macro_export]
macro_rules! receive_package_or_continue {
	($config:expr, $stream:expr, $accepted_actions:expr, $node:expr $(,)?) => {
		match common::package::Package::receive(
			$config,
			$stream,
			$accepted_actions,
		) {
			Ok(p) => p,
			Err(common::error::ReceivePackageError::InvalidAction) => {
				common::nprintln!(
					$node,
					"Received a package with invalid action."
				);
				continue;
			}
			Err(_) => {
				common::nprintln!($node, "Failed to receive a package.");
				continue;
			}
		}
	};
}

/// [`Send`](common::package::Package::send) a `common::package::Package`,
/// or `common::nprintln` error and `continue`.
#[macro_export]
macro_rules! send_package_or_continue {
	($config:expr, $package:expr, $stream:expr, $node:expr $(,)?) => {
		if $package.send($config, $stream).is_err() {
			common::nprintln!($node, "Failed to send a package.");
			continue;
		}
	};
}

/// [`Connect`](TcpStream::connect) to a `node`, or `common::nprintln` error
/// and `continue`.
#[macro_export]
macro_rules! connect_or_continue {
	($node:expr $(,)?) => {
		match std::net::TcpStream::connect($node) {
			Ok(s) => s,
			Err(_) => {
				common::nprintln!($node, "Failed to connect.");
				continue;
			}
		}
	};
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_set() {
		let s = set![1, 1, 2, 3, 4, 4, 5];
		assert_eq!(s.len(), 5);
	}
}
