use anyhow::Context;
use serde::Serialize;

use super::constants::{OPENID_AUTH_NAMESPACE, OPENID_PROVIDER_IDENTIFIER};
use super::xml_util::*;

const NAMESPACE_DEFAULT: &str = "xri://$xrd*($v*2.0)";
const NAMESPACE_XRDS: &str = "xri://$xrds";

const TAG_NAME_XRD: &str = "XRD";
const TAG_NAME_SERVICE: &str = "Service";
const TAG_NAME_TYPE: &str = "Type";
const TAG_NAME_URI: &str = "URI";

const EXPECTED_NAMESPACES: [Namespace; 2] = [
    Namespace::new(None, NAMESPACE_DEFAULT),
    Namespace::new(Some("xrds"), NAMESPACE_XRDS),
];

#[derive(Debug, Default, Clone, Serialize)]
pub(crate) struct Provider {
    pub(crate) version: String,
    pub(crate) endpoint: String,
}

#[cfg(test)]
impl Provider {
    pub(crate) fn steam() -> Provider {
        Provider {
            version: "http://specs.openid.net/auth/2.0/server".to_string(),
            endpoint: "https://steamcommunity.com/openid/login".to_string(),
        }
    }
}

/// TODO: Change this name so something descriptive
pub(crate) fn parse_xml(xml: &str) -> anyhow::Result<Provider> {
    let doc = roxmltree::Document::parse(xml).context("couldn't parse input document xml")?;
    let root = doc.root_element();

    namespaces_eq(&doc, &EXPECTED_NAMESPACES).context("namespaces validation failed")?;

    let xrd = get_only_child(&root, TAG_NAME_XRD)
        .context("get xrd element as only child of root element")?;
    let service = get_only_child(&xrd, TAG_NAME_SERVICE)
        .context("get service element as only child of xrd element")?;
    let service_children = get_child_set(&service, &[TAG_NAME_URI, TAG_NAME_TYPE])
        .context("get type and uri as only children of service element")?;

    let service_type = get_only_text_child(service_children.get(TAG_NAME_TYPE).unwrap())
        .context("couldn't get text of type element in service")?
        .to_string();

    // https://github.com/havard/node-openid/blob/672ea6e1b25e96c4a8e4f9deb74d38487c85ac32/openid.js#L287-L290
    if service_type.as_str() != OPENID_PROVIDER_IDENTIFIER {
        anyhow::bail!("text in type tag does not match spec");
    }

    let endpoint = get_only_text_child(service_children.get(TAG_NAME_URI).unwrap())
        .context("couldn't get text of uri element in service")?
        .to_string();

    Ok(Provider {
        endpoint,
        version: OPENID_AUTH_NAMESPACE.to_string(),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_steam_response() -> anyhow::Result<()> {
        const EXAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<xrds:XRDS xmlns:xrds="xri://$xrds" xmlns="xri://$xrd*($v*2.0)">
    <XRD>
        <Service priority="0">
            <Type>http://specs.openid.net/auth/2.0/server</Type>
            <URI>https://steamcommunity.com/openid/login</URI>
        </Service>
    </XRD>
</xrds:XRDS>"#;

        let service = parse_xml(EXAMPLE)?;
        assert_eq!(service.version, OPENID_AUTH_NAMESPACE);
        assert_eq!(service.endpoint, "https://steamcommunity.com/openid/login");

        Ok(())
    }
}
