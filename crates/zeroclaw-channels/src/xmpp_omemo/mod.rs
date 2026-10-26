//! Native legacy OMEMO with explicit identity pins and existing-state migration.
#![forbid(unsafe_code)]
pub mod bundle;
pub mod channel_state;
pub mod devices;
pub mod inbox;
pub mod initial_identity;
pub mod jid;
pub mod legacy;
pub mod omemo;
pub mod prekey_inventory;
pub mod prekeys;
pub mod private_store;
pub mod stanza;
pub mod store;
