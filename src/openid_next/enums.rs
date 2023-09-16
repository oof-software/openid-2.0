pub(crate) enum OpenIdUrl {
    IdentifierSelect,
    ReturnTo,
    Server,
    SignOn,
}
impl OpenIdUrl {
    pub(crate) const fn url(&self) -> &'static str {
        match self {
            OpenIdUrl::IdentifierSelect => "http://specs.openid.net/auth/2.0/identifier_select",
            OpenIdUrl::ReturnTo => "http://specs.openid.net/auth/2.0/return_to",
            OpenIdUrl::Server => "http://specs.openid.net/auth/2.0/server",
            OpenIdUrl::SignOn => "http://specs.openid.net/auth/2.0/signon",
        }
    }
}

pub(crate) enum OpenIdMode {
    Error,
    Associate,
    CheckIdImmediate,
    CheckIdSetup,
    IdentityResolution,
    SetupNeeded,
    Cancel,
    CheckAuthentication,
}
impl OpenIdMode {
    pub(crate) const fn value(&self) -> &'static str {
        match self {
            OpenIdMode::Error => "error",
            OpenIdMode::Associate => "associate",
            OpenIdMode::CheckIdImmediate => "checkid_immediate",
            OpenIdMode::CheckIdSetup => "checkid_setup",
            OpenIdMode::IdentityResolution => "id_res",
            OpenIdMode::SetupNeeded => "setup_needed",
            OpenIdMode::Cancel => "cancel",
            OpenIdMode::CheckAuthentication => "check_authentication",
        }
    }
}
