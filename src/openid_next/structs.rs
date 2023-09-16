/// All possible keys
pub(crate) struct OpenIdBase {
    pub(crate) assoc_handle: Option<String>,
    pub(crate) assoc_type: Option<String>,
    pub(crate) claimed_id: Option<String>,
    pub(crate) contact: Option<String>,
    pub(crate) delegate: Option<String>,
    pub(crate) dh_consumer_public: Option<String>,
    pub(crate) dh_gen: Option<String>,
    pub(crate) dh_modulus: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) identity: Option<String>,
    pub(crate) invalidate_handle: Option<String>,
    pub(crate) mode: Option<String>,
    pub(crate) ns: Option<String>,
    pub(crate) op_endpoint: Option<String>,
    pub(crate) openid: Option<String>,
    pub(crate) realm: Option<String>,
    pub(crate) reference: Option<String>,
    pub(crate) response_nonce: Option<String>,
    pub(crate) return_to: Option<String>,
    pub(crate) server: Option<String>,
    pub(crate) session_type: Option<String>,
    pub(crate) sig: Option<String>,
    pub(crate) signed: Option<String>,
    pub(crate) trust_root: Option<String>,
}

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.5.2.3>
pub(crate) struct IndirectErrorResponse {
    pub(crate) ns: String,
    pub(crate) mode: String,
    pub(crate) error: String,
    pub(crate) contact: Option<String>,
    pub(crate) reference: Option<String>,
}

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.9.1>
pub(crate) struct AuthenticationRequest {
    pub(crate) ns: String,
    pub(crate) mode: String,
    pub(crate) claimed_id: Option<String>,
    pub(crate) identity: Option<String>,
    pub(crate) assoc_handle: Option<String>,
    pub(crate) return_to: Option<String>,
    pub(crate) realm: Option<String>,
}

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.1>
pub(crate) struct PositiveAssertion {
    pub(crate) ns: String,
    pub(crate) mode: String,
    pub(crate) op_endpoint: String,
    pub(crate) claimed_id: Option<String>,
    pub(crate) identity: Option<String>,
    pub(crate) return_to: String,
    pub(crate) response_nonce: String,
    pub(crate) invalidate_handle: Option<String>,
    pub(crate) assoc_handle: String,
    pub(crate) signed: String,
    pub(crate) sig: String,
}

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.10.2.2>
pub(crate) struct NegativeAssertion {
    pub(crate) ns: String,
    pub(crate) mode: String,
}
