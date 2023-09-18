use std::str::FromStr;

use anyhow::Context;
use serde::Serialize;

use super::key_values::KeyValues;
use crate::openid::constants::OPENID_MODE_CHECK_AUTHENTICATION;
use crate::openid::{PositiveAssertion, Provider};

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.11.4.2.2>
#[derive(Debug, Default, Serialize)]
pub(crate) struct VerifyResponse {
    namespace: String,
    is_valid: bool,
}

impl TryFrom<&KeyValues> for VerifyResponse {
    type Error = anyhow::Error;
    fn try_from(kvs: &KeyValues) -> Result<Self, Self::Error> {
        let namespace = kvs
            .get("namespace")
            .context("missing field namespace")?
            .to_string();
        let is_valid = kvs
            .get("is_valid")
            .context("missing field is_valid")?
            .parse()
            .context("field is_valid does not contain a bool")?;
        Ok(VerifyResponse {
            namespace,
            is_valid,
        })
    }
}

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.11.4.2>
pub(crate) async fn verify_against_provider(
    client: &reqwest::Client,
    provider: &Provider,
    assertion: &PositiveAssertion,
) -> anyhow::Result<VerifyResponse> {
    let url = provider.service.endpoint.as_str();

    // https://github.com/havard/node-openid/blob/672ea6e1b25e96c4a8e4f9deb74d38487c85ac32/openid.js#L1250-L1253
    // https://openid.net/specs/openid-authentication-2_0.html#rfc.section.11.4.2.1
    let mut assertion = assertion.clone();
    assertion.set_mode(OPENID_MODE_CHECK_AUTHENTICATION);

    let req = client
        .post(url)
        .form(&assertion)
        .send()
        .await
        .context("couldn't send request to validate assertion")?;

    let text = req
        .text()
        .await
        .context("provider returned an invalid response")?;

    let key_values = KeyValues::from_str(&text)
        .context("couldn't parse response from provider as key-values")?;
    let verification = VerifyResponse::try_from(&key_values)
        .context("couldn't parse key-values as verification response")?;

    Ok(verification)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use anyhow::Context;

    use crate::openid::constants::OPENID_AUTH_NAMESPACE;
    use crate::openid::key_values::KeyValues;
    use crate::openid::VerifyResponse;

    #[test]
    fn key_value_deserialize() -> anyhow::Result<()> {
        const TEXT: &str = "ns:http://specs.openid.net/auth/2.0\nis_valid:true\n";

        let parsed = KeyValues::from_str(TEXT).context("invalid key values")?;
        let verification = VerifyResponse::try_from(&parsed).context("invalid response")?;

        assert_eq!(verification.is_valid, true);
        assert_eq!(verification.namespace, OPENID_AUTH_NAMESPACE);

        Ok(())
    }
}
