use anyhow::Context;

use super::constants::{OPENID_AUTH_NAMESPACE, OPENID_MODE_CHECK_AUTHENTICATION};
use super::{PositiveAssertion, Provider};

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.11.4.2.2>
#[derive(Debug, Default)]
pub(crate) struct VerifyResponse {
    namespace: String,
    is_valid: bool,
}

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.4.1.1>
///
/// <https://github.com/havard/node-openid/blob/672ea6e1b25e96c4a8e4f9deb74d38487c85ac32/openid.js#L220-L240>
fn parse_key_value_form(text: &str) -> anyhow::Result<VerifyResponse> {
    // Trimming the text is probebly illegal but whatever ¯\_(ツ)_/¯
    let lines = text.trim().split('\n');

    let kv_pairs: Option<Vec<_>> = lines.map(|line| line.split_once(':')).collect();
    let kv_pairs = kv_pairs.context("key value form contains a line without a semicolon")?;

    let mut response = VerifyResponse::default();
    for (k, v) in kv_pairs {
        match k {
            "ns" => {
                if v != OPENID_AUTH_NAMESPACE {
                    anyhow::bail!("response field ns contains an invalid value")
                }
                response.namespace = OPENID_AUTH_NAMESPACE.to_string();
            }
            "is_valid" => {
                response.is_valid = v
                    .parse()
                    .context("response field is_valid is not a valid bool")?
            }
            _ => anyhow::bail!("response contains unknown field {}", k),
        }
    }
    Ok(response)
}

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.11.4.2>
pub(crate) async fn verify_against_provider(
    client: &reqwest::Client,
    provider: &Provider,
    assertion: &PositiveAssertion,
) -> anyhow::Result<VerifyResponse> {
    let url = provider.endpoint.as_str();

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

    let parsed = parse_key_value_form(&text).context("couldn't parse response from provider")?;

    Ok(parsed)
}

#[cfg(test)]
mod test {
    use anyhow::Context;

    use crate::openid::{constants::OPENID_AUTH_NAMESPACE, validate::parse_key_value_form};

    #[test]
    fn key_value_deserialize() -> anyhow::Result<()> {
        const TEXT: &str = "ns:http://specs.openid.net/auth/2.0\nis_valid:true\n";

        let parsed = parse_key_value_form(TEXT).context("couldn't parse key value form")?;
        assert_eq!(parsed.is_valid, true);
        assert_eq!(parsed.namespace, OPENID_AUTH_NAMESPACE);

        Ok(())
    }
}
