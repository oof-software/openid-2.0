use anyhow::Context;

use super::constants::*;
use super::Provider;

/// Static params, missing `return_to` and `realm`.
///
/// See [`make_auth_req_params`].
const OPENID_STATIC_PARAMS: [Params<'static>; 4] = [
    // Not using immediate mode
    Params::new(OPENID_MODE, "checkid_setup"),
    // Using OpenID 2.0
    Params::new(OPENID_NAMESPACE, OPENID_AUTH_NAMESPACE),
    Params::new(OPENID_IDENTITY, OPENID_IDENTIFIER_SELECT),
    Params::new(OPENID_CLAIMED_ID, OPENID_IDENTIFIER_SELECT),
];

#[derive(Clone)]
pub(crate) struct Params<'a> {
    key: &'a str,
    value: &'a str,
}

impl<'a> Params<'a> {
    pub(crate) const fn new(key: &'a str, value: &'a str) -> Params<'a> {
        Params { key, value }
    }
    pub(crate) const fn into_pair(self) -> (&'a str, &'a str) {
        (self.key, self.value)
    }
}

/// Build the query string parameters to make the authentication request
///
/// # Example
///
/// All the parameters are static except `return_to` and `realm`.
///
/// ```json
/// {
///   "openid.mode": "checkid_setup",
///   "openid.ns": "http://specs.openid.net/auth/2.0",
///   "openid.identity": "http://specs.openid.net/auth/2.0/identifier_select",
///   "openid.claimed_id": "http://specs.openid.net/auth/2.0/identifier_select",
///   "openid.realm": "http://localhost:3000",
///   "openid.return_to": "http://localhost:3000/auth/steam/callback",
/// }
/// ```
fn make_auth_req_params<'a>(realm: &'a str, return_to: &'a str) -> Vec<Params<'a>> {
    let mut params = Vec::with_capacity(OPENID_STATIC_PARAMS.len() + 2);
    params.extend_from_slice(&OPENID_STATIC_PARAMS);
    params.push(Params::new(OPENID_REALM, realm));
    params.push(Params::new(OPENID_RETURN_TO, return_to));
    params
}

/// Build the url the user should be redirected to to authenticate.
///
/// See [`make_auth_req_params`]
pub(crate) fn make_auth_req_url(
    provider: &Provider,
    realm: &str,
    return_to: &str,
) -> anyhow::Result<String> {
    let return_to = reqwest::Url::parse(return_to).context("couldn't parse return_to url")?;
    let realm = reqwest::Url::parse(realm).context("couldn't parse realm url")?;

    let return_to_host = return_to
        .host_str()
        .context("return_to url is missing host part")?;
    let realm_host = return_to
        .host_str()
        .context("realm url is missing host part")?;

    if return_to_host != realm_host {
        anyhow::bail!("host part of realm and return_to urls don't match");
    }
    if return_to.scheme() != realm.scheme() {
        anyhow::bail!("scheme part of realm and return_to urls don't match");
    }

    let params = make_auth_req_params(realm.as_str(), return_to.as_str());
    let params: Vec<_> = params.into_iter().map(Params::into_pair).collect();

    let url = reqwest::Url::parse_with_params(&provider.service.endpoint, params)
        .context("couldn't parse provider endpoint with query params into a url")?;

    Ok(url.into())
}

#[cfg(test)]
mod test {
    use super::*;

    fn sorted_query_pairs(url: &str) -> anyhow::Result<(reqwest::Url, Vec<(String, String)>)> {
        let url = reqwest::Url::parse(url).context("couldn't parse url")?;
        let mut query: Vec<_> = url
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        query.sort_unstable_by(|(lhs, _), (rhs, _)| lhs.cmp(rhs));
        Ok((url, query))
    }

    #[test]
    fn test_make_auth_req_url() -> anyhow::Result<()> {
        const REALM: &str = "http://localhost:3000/";
        const RETURN_TO: &str = "http://localhost:3000/auth/steam/callback/";
        const EXPECTED_URL: &str = "https://steamcommunity.com/openid/login?openid.mode=checkid_setup&openid.ns=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0&openid.identity=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0%2Fidentifier_select&openid.claimed_id=http%3A%2F%2Fspecs.openid.net%2Fauth%2F2.0%2Fidentifier_select&openid.return_to=http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fsteam%2Fcallback%2F&openid.realm=http%3A%2F%2Flocalhost%3A3000%2F";

        let provider = Provider::steam();

        let url = make_auth_req_url(&provider, REALM, RETURN_TO)?;

        let (expected_url, expected_query) = sorted_query_pairs(EXPECTED_URL)?;
        let (url, query) = sorted_query_pairs(&url)?;

        assert_eq!(query.len(), expected_query.len());
        std::iter::zip(query.into_iter(), expected_query.into_iter()).for_each(
            |((actual_key, actual_value), (expected_key, expected_value))| {
                assert_eq!(
                    actual_key, expected_key,
                    "actual key (left) doesn't match expected (right)"
                );
                assert_eq!(
                    actual_value, expected_value,
                    "actual value (left) doesn't match expected (right) for keys actual `{}`, expected `{}`",
                    actual_key, expected_key
                );
            },
        );

        assert_eq!(url.origin(), expected_url.origin());
        Ok(())
    }
}
