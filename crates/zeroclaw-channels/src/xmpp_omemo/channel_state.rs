//! Native channel boundary for explicitly pinned legacy OMEMO sessions.
//! The caller resolves live allowlists and pins for every operation; no policy cache.
use crate::xmpp_omemo::{
    private_store::PrivateStore,
    stanza,
    store::{self, Inbound, Outbound, ReservedEnvelope},
};
use anyhow::{Context, Result, ensure};
use base64::{
    Engine,
    engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD},
};
use rusqlite::OptionalExtension;
use serde_json::Value;

fn is_authorized_pin(peer: &str, pin: [u8; 32], policy: &[String]) -> Result<bool> {
    let peer_jid: jid::BareJid = peer.parse()?;
    ensure!(
        peer_jid.node().is_some() && !peer.contains('/'),
        "Invalid peer JID"
    );
    let mut trusted = false;
    for entry in policy {
        let mut parts = entry.split_whitespace();
        let jid: jid::BareJid = parts
            .next()
            .context("Invalid fingerprint policy")?
            .parse()?;
        let fingerprint: String = parts.collect();
        let fingerprint = fingerprint
            .strip_prefix("05")
            .filter(|_| fingerprint.len() == 66)
            .unwrap_or(&fingerprint);
        ensure!(fingerprint.len() == 64, "Invalid fingerprint policy");
        let fingerprint = hex::decode(fingerprint)?;
        if jid == peer_jid && fingerprint == pin {
            trusted = true;
        }
    }
    Ok(trusted)
}
fn authorize_pin(peer: &str, pin: [u8; 32], policy: &[String]) -> Result<()> {
    ensure!(
        is_authorized_pin(peer, pin, policy)?,
        "Device identity is not currently pinned"
    );
    Ok(())
}
pub struct ChannelState {
    owner: PrivateStore,
    account: String,
    device: u32,
    max_chars: usize,
    cipher: std::sync::Arc<dyn crate::xmpp_omemo::inbox::InboxCipher>,
}
impl ChannelState {
    /// Automatic identity initialization is allowed only when the entire
    /// channel directory is absent, never when an existing identity is unreadable.
    pub fn open_or_initialize(
        path: &std::path::Path,
        account: &str,
        cipher: std::sync::Arc<dyn crate::xmpp_omemo::inbox::InboxCipher>,
        max_chars: usize,
    ) -> Result<Self> {
        ensure!(
            (1..=64000).contains(&max_chars),
            "Invalid ingress text limit"
        );
        cipher.check_existing()?;
        match std::fs::symlink_metadata(path) {
            Ok(_) => Self::open_existing(path, account, cipher, max_chars),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let mut owner = PrivateStore::create_new(path)?;
                let mut random = [0u8; 4];
                getrandom::fill(&mut random).map_err(|_| {
                    ::zeroclaw_log::record!(
                        ERROR,
                        ::zeroclaw_log::Event::new(module_path!(), ::zeroclaw_log::Action::Fail),
                        "Device identity randomness unavailable"
                    );
                    anyhow::Error::msg("Device identity randomness unavailable")
                })?;
                let device = (u32::from_le_bytes(random) & i32::MAX as u32).max(1);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();
                crate::xmpp_omemo::initial_identity::initialize_empty(
                    owner.connection(),
                    account,
                    device,
                    now,
                )?;
                drop(owner);
                Self::open_existing(path, account, cipher, max_chars)
            }
            Err(error) => Err(error.into()),
        }
    }
    pub fn open_existing(
        path: &std::path::Path,
        account: &str,
        cipher: std::sync::Arc<dyn crate::xmpp_omemo::inbox::InboxCipher>,
        max_chars: usize,
    ) -> Result<Self> {
        ensure!(
            (1..=64000).contains(&max_chars),
            "Invalid ingress text limit"
        );
        cipher.check_existing()?;
        let mut owner = PrivateStore::open_existing(path)?;
        let tx = owner.connection().transaction()?;
        let (_, _, device) = store::identity(&tx, account)?;
        tx.commit()?;
        store::initialize_outbox(owner.connection(), account)?;
        crate::xmpp_omemo::inbox::initialize(owner.connection())?;
        Ok(Self {
            owner,
            account: account.into(),
            device,
            max_chars,
            cipher,
        })
    }
    fn pin(&mut self, peer: &str, device: u32, policy: &[String]) -> Result<[u8; 32]> {
        ensure!((1..=i32::MAX as u32).contains(&device), "Invalid device ID");
        let prefix = format!("omemo//eu.siacs.conversations.axolotl/{peer}/{device}/");
        let tx = self.owner.connection().transaction()?;
        let associated = store::load(&tx, &format!("{prefix}associated_data"))?;
        let encoded = associated
            .as_str()
            .context("Missing associated identities")?;
        let bytes = URL_SAFE
            .decode(encoded)
            .or_else(|_| URL_SAFE_NO_PAD.decode(encoded))?;
        ensure!(
            bytes.len() == 66 && bytes[0] == 5 && bytes[33] == 5,
            "Invalid associated identities"
        );
        let role = store::load(&tx, &format!("{prefix}initiation"))?;
        let remote = match role.as_str() {
            Some("ACTIVE") => &bytes[34..66],
            Some("PASSIVE") => &bytes[1..33],
            _ => anyhow::bail!("Unknown session role"),
        };
        let pin: [u8; 32] = remote.try_into()?;
        authorize_pin(peer, pin, policy)?;
        tx.commit()?;
        Ok(pin)
    }
    pub fn public_bundle(&mut self) -> Result<crate::xmpp_omemo::bundle::PublicBundle> {
        crate::xmpp_omemo::prekey_inventory::replenish(
            self.owner.connection(),
            &self.account,
            100,
        )?;
        let bundle =
            crate::xmpp_omemo::bundle::export_existing(self.owner.connection(), &self.account)?;
        ensure!(
            bundle.prekey_count >= 100,
            "Incomplete native prekey inventory"
        );
        Ok(bundle)
    }
    /// A missing ratchet never permits overwriting partial existing metadata.
    pub fn discovery_request(
        &mut self,
        peer: &str,
        xml: &str,
        policy: &[String],
    ) -> Result<Option<(u32, [u8; 32])>> {
        let selected = stanza::parse(xml, self.device)?;
        if selected.ciphertext.is_none() {
            return Ok(None);
        }
        let prefix = format!(
            "omemo//eu.siacs.conversations.axolotl/{peer}/{}/",
            selected.sender_device
        );
        let exists: bool = self.owner.connection().query_row(
            "SELECT EXISTS(SELECT 1 FROM state WHERE key=?1)",
            [format!("{prefix}double_ratchet")],
            |r| r.get(0),
        )?;
        if exists {
            return Ok(None);
        }
        let keys: Vec<String> = self
            .owner
            .connection()
            .prepare("SELECT key FROM state")?
            .query_map([], |r| r.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        ensure!(
            !keys.iter().any(|key| key.starts_with(&prefix)),
            "Partial session metadata must not be replaced"
        );
        ensure!(selected.prekey, "First contact requires a prekey exchange");
        let exchange =
            crate::xmpp_omemo::omemo::wire::PreKeySignalMessage::deserialize(&selected.wire)
                .context("Invalid first-contact exchange")?;
        let pin: [u8; 32] = exchange.identity_key.as_slice().try_into()?;
        authorize_pin(peer, pin, policy)?;
        Ok(Some((selected.sender_device, pin)))
    }
    pub fn receive_first(
        &mut self,
        peer: &str,
        id: &str,
        xml: &str,
        bundle_xml: &str,
        policy: &[String],
    ) -> Result<Option<String>> {
        let (device, pin) = self
            .discovery_request(peer, xml, policy)?
            .context("First contact no longer eligible")?;
        let bundle = crate::xmpp_omemo::bundle::parse_pinned(bundle_xml, device, pin)?;
        let sign = bundle.signed_pre_key.signature[63] >> 7;
        let ed = curve25519_dalek::montgomery::MontgomeryPoint(pin)
            .to_edwards(sign)
            .context("Invalid peer identity")?
            .compress()
            .to_bytes();
        let selected = stanza::parse(xml, self.device)?;
        let payload = selected
            .ciphertext
            .context("Missing first-contact payload")?;
        let cipher = self.cipher.clone();
        let account = self.account.clone();
        let max_chars = self.max_chars;
        let plaintext = crate::xmpp_omemo::prekeys::receive_fresh_with(
            self.owner.connection(),
            Inbound {
                account: &self.account,
                peer,
                peer_device: device,
                pinned_fingerprint: pin,
                stanza_id: id,
                wire: &selected.wire,
                ciphertext: &payload,
                nonce: &selected.nonce,
            },
            ed,
            |tx, content| {
                let text = std::str::from_utf8(content)?;
                ensure!(
                    !text.trim().is_empty() && text.chars().count() <= max_chars,
                    "Invalid ingress text length"
                );
                crate::xmpp_omemo::inbox::stage(
                    tx, &cipher, &account, peer, id, device, pin, content,
                )
            },
        )?;
        Ok(Some(String::from_utf8(plaintext)?))
    }
    pub fn receive_existing(
        &mut self,
        peer: &str,
        id: &str,
        xml: &str,
        policy: &[String],
    ) -> Result<Option<String>> {
        ensure!(!id.is_empty() && id.len() <= 512, "Invalid stanza ID");
        let seen: bool = self.owner.connection().query_row(
            "SELECT EXISTS(SELECT 1 FROM seen WHERE sender=?1 AND id=?2)",
            rusqlite::params![peer, id],
            |r| r.get(0),
        )?;
        if seen {
            return Ok(None);
        }
        let selected = stanza::parse(xml, self.device)?;
        let Some(payload) = selected.ciphertext else {
            return Ok(None);
        };
        let pin = self.pin(peer, selected.sender_device, policy)?;
        let cipher = self.cipher.clone();
        let account = self.account.clone();
        let max_chars = self.max_chars;
        let plaintext = store::receive_with(
            self.owner.connection(),
            Inbound {
                account: &self.account,
                peer,
                peer_device: selected.sender_device,
                pinned_fingerprint: pin,
                stanza_id: id,
                wire: &selected.wire,
                ciphertext: &payload,
                nonce: &selected.nonce,
            },
            |tx, content| {
                let text = std::str::from_utf8(content)?;
                ensure!(
                    !text.trim().is_empty() && text.chars().count() <= max_chars,
                    "Invalid ingress text length"
                );
                crate::xmpp_omemo::inbox::stage(
                    tx,
                    &cipher,
                    &account,
                    peer,
                    id,
                    selected.sender_device,
                    pin,
                    content,
                )
            },
        )?;
        Ok(Some(String::from_utf8(plaintext)?))
    }
    pub fn pending(&mut self, policy: &[String]) -> Result<Vec<crate::xmpp_omemo::inbox::Pending>> {
        let messages = crate::xmpp_omemo::inbox::pending(
            self.owner.connection(),
            &self.cipher,
            &self.account,
        )?;
        let mut allowed = Vec::new();
        for msg in messages {
            if msg.content.trim().is_empty() || msg.content.chars().count() > self.max_chars {
                continue;
            }
            if is_authorized_pin(&msg.peer, msg.pin, policy)? {
                ensure!(
                    self.pin(&msg.peer, msg.device, policy)? == msg.pin,
                    "Pending ingress fingerprint changed"
                );
                allowed.push(msg);
            }
        }
        Ok(allowed)
    }
    pub fn admit(&mut self, peer: &str, id: &str, policy: &[String]) -> Result<bool> {
        let phase: String = self.owner.connection().query_row(
            "SELECT phase FROM native_omemo_inbox WHERE peer=?1 AND id=?2",
            rusqlite::params![peer, id],
            |r| r.get(0),
        )?;
        if phase == "pending" {
            ensure!(
                self.pending(policy)?
                    .iter()
                    .any(|p| p.peer == peer && p.id == id),
                "Ingress identity is no longer authorized"
            );
        }
        crate::xmpp_omemo::inbox::admit(self.owner.connection(), peer, id)
    }
    pub fn bind_runtime(
        &mut self,
        peer: &str,
        id: &str,
        agent: &str,
        history_key: &str,
        policy: &[String],
    ) -> Result<()> {
        let record = crate::xmpp_omemo::inbox::inspect(
            self.owner.connection(),
            &self.cipher,
            &self.account,
            peer,
            id,
        )?;
        ensure!(
            is_authorized_pin(peer, record.pin, policy)?
                && self.pin(peer, record.device, policy)? == record.pin,
            "Ingress identity is no longer authorized"
        );
        crate::xmpp_omemo::inbox::bind_runtime(
            self.owner.connection(),
            &self.cipher,
            &self.account,
            peer,
            id,
            crate::xmpp_omemo::inbox::RuntimeLink {
                agent: agent.into(),
                history_key: history_key.into(),
            },
        )
    }
    pub fn returned(&mut self, peer: &str, id: &str) -> Result<()> {
        crate::xmpp_omemo::inbox::returned(self.owner.connection(), peer, id)
    }
    pub fn reserve_existing(
        &mut self,
        peer: &str,
        id: &str,
        plaintext: &[u8],
        policy: &[String],
    ) -> Result<ReservedEnvelope> {
        let key = format!("omemo//eu.siacs.conversations.axolotl/{peer}/device_ids");
        let raw: Option<String> = self
            .owner
            .connection()
            .query_row("SELECT value FROM state WHERE key=?1", [key], |r| r.get(0))
            .optional()?;
        let value: Value = serde_json::from_str(&raw.context("No existing peer device index")?)?;
        let devices = value.as_array().context("Invalid peer device index")?;
        ensure!(
            !devices.is_empty() && devices.len() <= 256,
            "Invalid peer device count"
        );
        let mut peers = Vec::new();
        for device in devices {
            let device = u32::try_from(device.as_u64().context("Invalid device index")?)?;
            let pin = self.pin(peer, device, policy)?;
            peers.push((device, pin));
        }
        store::reserve_send_many(
            self.owner.connection(),
            Outbound {
                account: &self.account,
                peer,
                peer_device: peers[0].0,
                pinned_fingerprint: peers[0].1,
                stanza_id: id,
                plaintext,
            },
            &peers,
        )
    }
    pub fn complete_socket_write(&mut self, envelope: &ReservedEnvelope) -> Result<()> {
        store::complete_socket_write(self.owner.connection(), &self.account, envelope)
    }
}

#[cfg(test)]
mod policy_tests {
    use super::is_authorized_pin;
    #[test]
    fn configured_grouping_and_legacy_key_prefix_match_native_policy() -> anyhow::Result<()> {
        let pin = [0xabu8; 32];
        for entry in [
            format!("alice@example.test {}", hex::encode(pin)),
            format!(
                "  alice@example.test  05 {}  ",
                hex::encode_upper(pin)
                    .as_bytes()
                    .chunks(8)
                    .map(|p| std::str::from_utf8(p).unwrap())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ] {
            assert!(is_authorized_pin("alice@example.test", pin, &[entry])?);
        }
        assert!(!is_authorized_pin(
            "alice@example.test",
            pin,
            &[format!("mallory@example.test {}", hex::encode(pin))]
        )?);
        assert!(
            is_authorized_pin(
                "alice@example.test",
                pin,
                &[format!("alice@example.test/resource {}", hex::encode(pin))]
            )
            .is_err()
        );
        assert!(
            is_authorized_pin(
                "alice@example.test",
                pin,
                &["alice@example.test 05ab".into()]
            )
            .is_err()
        );
        Ok(())
    }
}
