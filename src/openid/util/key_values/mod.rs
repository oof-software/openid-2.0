//! - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.4.1.1>
//! - <https://github.com/havard/node-openid/blob/672ea6e1b25e96c4a8e4f9deb74d38487c85ac32/openid.js#L220-L240>
//! - <https://doc.rust-lang.org/std/primitive.str.html#method.parse>
//! - <https://serde.rs/data-format.html>
//! - <https://docs.rs/serde/1.0.188/serde/macro.forward_to_deserialize_any.html>
//! - <https://durch.github.io/rust-goauth/serde_urlencoded/index.html>
//! - <https://durch.github.io/rust-goauth/src/serde_urlencoded/de.rs.html>
//!
//! See test cases in `de.rs`.
//!
//! # ToDo
//!
//! - A lot probably
//!
//! # Example
//!
//! Note: The trailing newline is mandatory!
//!
//! ```text
//! keyOne:valueOne\nkeyTwo:value:Two\n
//! ```
//!
//! Is parsed as
//!
//! ```json
//! { "keyOne": "valueOne", "keyTwo": "value:Two" }
//! ```

mod de;
pub use de::{from_str, Error};
