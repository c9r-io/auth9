//! Argon2id password hashing with OWASP-recommended parameters.

use argon2::{Algorithm, Argon2, Params, Version};

/// Returns an Argon2id instance configured with OWASP-recommended parameters:
/// - memory: 65536 KB (64 MB)
/// - iterations: 3
/// - parallelism: 4
///
/// Use this for all production password/secret hashing operations.
/// For **verification**, `Argon2::default()` is acceptable because the
/// parameters are read from the stored hash string.
pub fn owasp_argon2() -> Argon2<'static> {
    let params = Params::new(65536, 3, 4, None).expect("valid OWASP argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}
