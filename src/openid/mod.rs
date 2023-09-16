//! # Terminology
//!
//! ## **Identifier**
//!
//! An Identifier is either a "http" or "https" URI, (commonly referred to as a "URL" within this document), or an XRI. This document defines various kinds of Identifiers, designed for use in different contexts.
//!
//! ## **User-Agent**
//!
//! The end user's Web browser which implements HTTP/1.1.
//!
//! ## **Relying Party** (**RP**)
//!
//! A Web application that wants proof that the end user controls an Identifier.
//!
//! ## **OpenID Provider** (**OP**)
//!
//! An OpenID Authentication server on which a Relying Party relies for an assertion that the end user controls an Identifier.
//!
//! ## **OP Endpoint URL**
//!
//! The URL which accepts OpenID Authentication protocol messages, obtained by performing discovery on the User-Supplied Identifier. This value MUST be an absolute HTTP or HTTPS URL.
//!
//! ## **OP Identifier**
//!
//! An Identifier for an OpenID Provider.
//!
//! ## **User-Supplied Identifier**
//!
//! An Identifier that was presented by the end user to the Relying Party, or selected by the user at the OpenID Provider. During the initiation phase of the protocol, an end user may enter either their own Identifier or an OP Identifier. If an OP Identifier is used, the OP may then assist the end user in selecting an Identifier to share with the Relying Party.
//!
//! ## **Claimed Identifier**
//!
//! An Identifier that the end user claims to own; the overall aim of the protocol is verifying this claim. The Claimed Identifier is either
//!
//! - The Identifier obtained by normalizing the User-Supplied Identifier, if it was an URL.
//! - The CanonicalID, if it was an XRI.
//!
//! ## **OP-Local Identifier**
//!
//! An alternate Identifier for an end user that is local to a particular OP and thus not necessarily under the end user's control.

pub(crate) mod constants;
mod params;
mod response;
mod validate;
mod xml;
mod xml_util;

pub(crate) use params::*;
pub(crate) use response::*;
pub(crate) use validate::*;
pub(crate) use xml::*;
