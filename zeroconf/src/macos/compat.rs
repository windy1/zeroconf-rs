//! Utilities related to compatibility between platforms

/// Normalizes the specified domain `&str` to conform to a standard enforced by this crate.
///
/// Bonjour suffixes domains with a final `'.'` character in some contexts but is not required by
/// the standard. This function removes the final dot if present.
pub fn normalize_domain(domain: &str) -> String {
    if domain.chars().nth(domain.len() - 1).unwrap() == '.' {
        String::from(&domain[..domain.len() - 1])
    } else {
        String::from(domain)
    }
}
