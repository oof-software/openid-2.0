use anyhow::Context;
use roxmltree::Node;
use serde::Serialize;

use crate::openid::constants::{
    OPENID_AUTH_NAMESPACE, OPENID_PRIORITY_ATTRIBUTE, OPENID_PROVIDER_IDENTIFIER,
};
use crate::openid::util::xml::*;

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

/// <https://openid.net/specs/openid-authentication-2_0.html#rfc.section.7.3.2.1.2>
///
/// A Claimed Identifier Element is an `<xrd:Service>` element with the following information:
/// - An `<xrd:Type>` tag whose text content is `http://specs.openid.net/auth/2.0/signon`.
/// - An `<xrd:URI>` tag whose text content is the OP Endpoint URL.
/// - An `<xrd:LocalID>` tag (optional) whose text content is the OP-Local Identifier.
#[derive(Debug, Default, Clone, Serialize)]
pub(crate) struct Service {
    pub(crate) version: String,
    pub(crate) endpoint: String,
    pub(crate) local_id: Option<String>,
    pub(crate) priority: Option<i32>,
}

impl Service {
    fn from_node(service_node: Node) -> anyhow::Result<Service> {
        if service_node.tag_name().name() != TAG_NAME_SERVICE {
            anyhow::bail!("trying to parse service element with invalid tag name");
        }

        let Some(priority) = service_node.attribute(OPENID_PRIORITY_ATTRIBUTE) else {
            anyhow::bail!("service element is missing priority attribute");
        };
        let priority = priority
            .parse()
            .context("couldn't parse priority as an integer")?;

        let service_children = get_child_set(service_node, &[TAG_NAME_URI, TAG_NAME_TYPE])
            .context("get type and uri as only children of service element")?;

        let service_type = get_only_text_child(*service_children.get(TAG_NAME_TYPE).unwrap())
            .context("couldn't get text of type element in service")?
            .to_string();

        // https://github.com/havard/node-openid/blob/672ea6e1b25e96c4a8e4f9deb74d38487c85ac32/openid.js#L287-L290
        if service_type.as_str() != OPENID_PROVIDER_IDENTIFIER {
            anyhow::bail!("text in type tag does not match spec");
        }

        let endpoint = get_only_text_child(*service_children.get(TAG_NAME_URI).unwrap())
            .context("couldn't get text of uri element in service")?
            .to_string();

        Ok(Service {
            endpoint,
            version: OPENID_AUTH_NAMESPACE.to_string(),
            local_id: None,
            priority: Some(priority),
        })
    }
}

pub(crate) struct Provider {
    // TODO: This should be a `Vec<Service>` as a provider can expose
    //       multiple services and we should select them by their priority
    pub(crate) service: Service,
}

impl Provider {
    fn from_node(xrd_node: Node) -> anyhow::Result<Provider> {
        if xrd_node.tag_name().name() != TAG_NAME_XRD {
            anyhow::bail!("trying to parse provider element with invalid tag name");
        }

        let service_node = get_only_child(xrd_node, TAG_NAME_SERVICE)
            .context("get service element as only child of xrd element")?;

        Ok(Provider {
            service: Service::from_node(service_node)?,
        })
    }
    pub(crate) fn from_xml(xml: &str) -> anyhow::Result<Provider> {
        let doc = roxmltree::Document::parse(xml).context("couldn't parse input document xml")?;

        namespaces_eq(&doc, &EXPECTED_NAMESPACES).context("namespaces validation failed")?;

        let root_node = doc.root_element();

        let xrd_node = get_only_child(root_node, TAG_NAME_XRD)
            .context("get xrd element as only child of root element")?;

        Provider::from_node(xrd_node)
    }
}

impl Provider {
    #[cfg(test)]
    pub(crate) fn steam() -> Provider {
        let service = Service {
            version: "http://specs.openid.net/auth/2.0/server".to_string(),
            endpoint: "https://steamcommunity.com/openid/login".to_string(),
            local_id: None,
            priority: Some(0),
        };
        Provider { service }
    }
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

        let provider = Provider::from_xml(EXAMPLE)?;
        let service = provider.service;

        assert_eq!(service.version, OPENID_AUTH_NAMESPACE);
        assert_eq!(service.endpoint, "https://steamcommunity.com/openid/login");
        assert_eq!(service.local_id, None);
        assert_eq!(service.priority, Some(0));

        Ok(())
    }
}
