//! Mail sent from a linked mailbox, and what came back.
//!
//! A connector whose consent included sending carries a party's mail: the
//! monthly bundle to the accountant, mostly. Nothing here talks to a
//! provider directly; the kind does ([`crate::connectors::Connector::send`]),
//! and this module keeps the record. Every mail sent is stored verbatim with
//! its recipients and attachments; a reply in the same thread is imported on
//! the next pull, and its attachments become documents. A template holds
//! the recurring shape with `{{Month}}`-style helpers the page renders; the
//! service stores what was actually sent, never the template.
//!
//! Safety: `[mail] allow_to` in the configuration, when set, is the only
//! set of addresses a mail may go to, and the refusal comes before the
//! provider is asked. The local environment sets it to the owner's own two
//! addresses.

pub mod store;
