/// `openid.ns` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.4.1.2>
///
/// This particular value MUST be present for the request to be a valid OpenID Authentication 2.0 request.
/// Future versions of the specification may define different values in order to allow message recipients to properly interpret the request.
///
/// Value: `http://specs.openid.net/auth/2.0`
pub(crate) const OPENID_NAMESPACE: &str = "openid.ns";

/// See [`OPENID_NAMESPACE`]
pub(crate) const OPENID_NAMESPACE_2_0: &str = "http://specs.openid.net/auth/2.0";

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

/// `openid.mode` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.9.1>
///
/// If the Relying Party wishes the end user to be able to interact with the OP, `checkid_setup` should be used.
///
/// Value: `checkid_immediate` or `checkid_setup`
pub(crate) const OPENID_MODE: &str = "openid.mode";

/// See [`OPENID_MODE`]
pub(crate) const OPENID_MODE_IMMEDIATE: &str = "checkid_immediate";

/// See [`OPENID_MODE`]
pub(crate) const OPENID_MODE_CHECKID_SETUP: &str = "checkid_setup";

/// `openid.return_to` <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.9.1>
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
pub(crate) const OPENID_IDENT_ELEMENT_TYPE: &str = "http://specs.openid.net/auth/2.0/server";
