use std::collections::HashMap;

use anyhow::Context;
use roxmltree::{Document, Node};

#[derive(Clone)]
pub(crate) struct Namespace<'a> {
    name: Option<&'a str>,
    uri: &'a str,
}

impl<'a> Namespace<'a> {
    pub(crate) const fn new(name: Option<&'a str>, uri: &'a str) -> Namespace<'a> {
        Namespace { name, uri }
    }
    pub(crate) fn matches(&self, other: &roxmltree::Namespace) -> bool {
        self.name.eq(&other.name()) && self.uri.eq(other.uri())
    }
}

/// Sort namespaces by their names and then compare them
///
/// Allocates O(n) memory where n is the maximum number of namespaces
pub(crate) fn namespaces_eq(doc: &Document, namespaces: &[Namespace]) -> anyhow::Result<()> {
    let root = doc.root_element();
    let mut root_namespaces = root.namespaces().collect::<Vec<_>>();
    root_namespaces.sort_unstable_by(|lhs, rhs| lhs.name().cmp(&rhs.name()));

    let mut namespaces = namespaces.to_vec();
    namespaces.sort_unstable_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    if root_namespaces.len() != namespaces.len() {
        anyhow::bail!("root node doesn't have expected number of namespaces");
    }
    if !std::iter::zip(root_namespaces.iter(), namespaces.iter())
        .all(|(&root_ns, ns)| ns.matches(root_ns))
    {
        anyhow::bail!("at least one namespace differs from the expected");
    }

    Ok(())
}

/// Check that
/// - the node has at most one child with the given tag name
/// and return it
pub(crate) fn get_child_opt<'a, 'input>(
    node: Node<'a, 'input>,
    tag_name: &str,
) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|c| c.is_element() && c.tag_name().name() == tag_name)
}

/// Check that
/// - the node has exactly one child and
/// - the child has the given tag name
/// and return it
pub(crate) fn get_only_child<'a, 'input>(
    node: Node<'a, 'input>,
    tag_name: &str,
) -> anyhow::Result<Node<'a, 'input>> {
    let mut children = node.children().filter(|c| c.is_element());
    let first = children.next().context("node doesn't have any children")?;
    if children.next().is_some() {
        anyhow::bail!("node has more than one child");
    }
    if first.tag_name().name() != tag_name {
        anyhow::bail!("child has unexpected tag name");
    }
    Ok(first)
}

/// Check that
/// - all children have the given tag name
/// and return then
pub(crate) fn get_children_exact<'a, 'input>(
    node: Node<'a, 'input>,
    tag_name: &str,
) -> anyhow::Result<Vec<Node<'a, 'input>>> {
    let children = node.children().filter(|c| c.is_element());
    let mut buffer = Vec::new();

    for child in children {
        if child.tag_name().name() != tag_name {
            anyhow::bail!("node has a child with an unexpected tag name");
        }
        buffer.push(child);
    }

    Ok(buffer)
}

/// Check that
/// - the node has exactly one text child
/// and return that one.
pub(crate) fn get_only_text_child<'a>(node: Node<'a, '_>) -> anyhow::Result<&'a str> {
    let mut children = node.children().filter(|c| c.is_text());
    let first = children.next().context("node doesn't have any children")?;
    if children.next().is_some() {
        anyhow::bail!("node has more than one child");
    }
    let text = first
        .text()
        .context("node is only text child but it is empty")?;

    Ok(text)
}

/// Check that the node has exactly the children with given tag names, not more and not less.
pub(crate) fn get_child_set<'a, 'input, 'str>(
    node: Node<'a, 'input>,
    tag_names: &[&'str str],
) -> anyhow::Result<HashMap<&'str str, Node<'a, 'input>>> {
    let mut children_with_names = node
        .children()
        .filter(|c| c.is_element())
        .map(|c| (c.tag_name().name(), c))
        .collect::<Vec<_>>();

    if children_with_names.len() != tag_names.len() {
        anyhow::bail!("node has unexpected number of children");
    }
    children_with_names.sort_unstable_by_key(|c| c.0);

    let mut tag_names = tag_names.to_vec();
    tag_names.sort_unstable();

    if !std::iter::zip(children_with_names.iter(), tag_names.iter())
        .all(|((child_tag, _), expected_tag)| *child_tag == *expected_tag)
    {
        anyhow::bail!("tag names of children do not match expected tag names");
    }

    let map = std::iter::zip(children_with_names, tag_names)
        .map(|((_, child), tag)| (tag, child))
        .collect::<HashMap<_, _>>();

    Ok(map)
}
