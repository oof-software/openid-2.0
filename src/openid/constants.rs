#![allow(dead_code)]

/// `openid.ns` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.4.1.2>
///
/// This particular value MUST be present for the request to be a valid OpenID Authentication 2.0 request.
/// Future versions of the specification may define different values in order to allow message recipients to properly interpret the request.
///
/// Value: `http://specs.openid.net/auth/2.0`
pub(crate) const OPENID_NAMESPACE: &str = "openid.ns";

/// See [`OPENID_NAMESPACE`]
pub(crate) const OPENID_AUTH_NAMESPACE: &str = "http://specs.openid.net/auth/2.0";

/// See [`OPENID_IDENTITY`]
pub(crate) const OPENID_CLAIMED_ID: &str = "openid.claimed_id";

/// `openid.identity` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.9.1>
///
/// If a different OP-Local Identifier is not specified, the claimed identifier MUST be used as the value for `openid.identity`.
///
/// If this is set to the special value `http://specs.openid.net/auth/2.0/identifier_select`
/// then the OP SHOULD choose an Identifier that belongs to the end user.
pub(crate) const OPENID_IDENTITY: &str = "openid.identity";

/// See [`OPENID_IDENTITY`]
pub(crate) const OPENID_IDENTIFIER_SELECT: &str =
    "http://specs.openid.net/auth/2.0/identifier_select";

/// `openid.mode`
/// - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.5.2.3>
/// - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.8.1>
/// - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.9.1>
/// - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.9.3>
/// - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
/// - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.2.1>
/// - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.2.2>
/// - <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.11.4.2.1>
///
/// If the Relying Party wishes the end user to be able to interact with the OP, `checkid_setup` should be used.
///
/// Value: `checkid_immediate`, `checkid_setup` or `id_res`
pub(crate) const OPENID_MODE: &str = "openid.mode";

/// See [`OPENID_MODE`]
pub(crate) const OPENID_MODE_CHECKID_IMMEDIATE: &str = "checkid_immediate";

/// See [`OPENID_MODE`]
pub(crate) const OPENID_MODE_CHECKID_SETUP: &str = "checkid_setup";

/// See [`OPENID_MODE`]
pub(crate) const OPENID_MODE_IDENTIFIER_RESPONSE: &str = "id_res";

/// See [`OPENID_MODE`]
pub(crate) const OPENID_MODE_CHECK_AUTHENTICATION: &str = "check_authentication";

/// See [`OPENID_MODE`]
pub(crate) const OPENID_MODE_ERROR: &str = "error";

/// `openid.return_to` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.9.1> and
/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
///
/// URL to which the OP SHOULD return the User-Agent with the response indicating the status of the request.
pub(crate) const OPENID_RETURN_TO: &str = "openid.return_to";

/// `openid.realm` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.9.1>
///
/// URL pattern the OP SHOULD ask the end user to trust.
pub(crate) const OPENID_REALM: &str = "openid.realm";

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.7.3.2.1.1>
///
/// An OP Identifier Element is an <xrd:Service> element with the following information:
/// - An `<xrd:Type>` tag whose text content is `http://specs.openid.net/auth/2.0/server`.
/// - An `<xrd:URI>` tag whose text content is the OP Endpoint URL
pub(crate) const OPENID_PROVIDER_IDENTIFIER: &str = "http://specs.openid.net/auth/2.0/server";

/// `openid.op_endpoint` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
///
/// The OP Endpoint URL.
pub(crate) const OPENID_OP_ENDPOINT: &str = "openid.op_endpoint";

/// `openid.response_nonce` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
///
/// A string 255 characters or less in length, that MUST be unique to this particular successful authentication response.
///
/// The nonce MUST start with the current time on the server,
/// and MAY contain additional ASCII characters in the range 33-126 inclusive (printable non-whitespace characters).
///
/// The date and time MUST be formatted as specified in section 5.6 of [RFC3339], with the following restrictions:
/// - All times must be in the UTC timezone, indicated with a `Z`
/// - No fractional seconds are allowed
///
/// Example: `2005-05-15T17:11:51ZUNIQUE`
pub(crate) const OPENID_RESPONSE_NONCE: &str = "openid.response_nonce";

/// `openid.invalidate_handle` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
pub(crate) const OPENID_INVALIDATE_HANDLE: &str = "openid.invalidate_handle";

/// `openid.assoc_handle` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
///
/// The handle for the association that was used to sign this assertion.
pub(crate) const OPENID_ASSOCIATION_HANDLE: &str = "openid.assoc_handle";

/// `openid.signed` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
///
/// Comma-separated list of signed fields.
///
/// This entry consists of the fields without the "openid." prefix that the signature covers.
///
/// This list MUST contain at least
/// - `op_endpoint`
/// - `return_to`
/// - `response_nonce`
/// - `assoc_handle`
///
/// and if present in the response
/// - `claimed_id`
/// - `identity`
pub(crate) const OPENID_SIGNED_FIELDS: &str = "openid.signed";

/// `openid.sig` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
///
/// Base 64 encoded signature.
pub(crate) const OPENID_SIGNATURE: &str = "openid.sig";

/// See [`OPENID_RESPONSE_NONCE`]
pub(crate) const OPENID_RESPONSE_NONCE_MAX_LEN: usize = 255;

pub(crate) const OPENID_FIELD_PREFIX: &str = "openid.";

/// <http://docs.oasis-open.org/xri/2.0/specs/cd02/xri-resolution-V2.0-cd-02.html#_Ref124065812>
pub(crate) const OPENID_PRIORITY_ATTRIBUTE: &str = "priority";
