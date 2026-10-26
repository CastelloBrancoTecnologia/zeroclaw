//! Offline isolated administrative example. No agent/channel tool registration.
use super::vault::VaultInboxCipher;
use crate::xmpp_omemo::{
    inbox::{self, InboxCipher, Phase},
    private_store::PrivateStore,
};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::{os::unix::fs::MetadataExt, path::Path, sync::Arc};
pub fn run(args: &[String]) -> Result<Value> {
    ensure!(
        matches!(
            args.first().map(String::as_str),
            Some("inspect" | "inspect-release" | "release-returned")
        ),
        "Unknown offline ingress operation"
    );
    let operation = &args[0];
    ensure!(
        args.len()
            == if operation == "release-returned" {
                9
            } else {
                7
            },
        "Invalid offline ingress arguments"
    );
    let root = Path::new(&args[1]);
    let state = Path::new(&args[2]);
    let alias = &args[3];
    let account = &args[4];
    let peer = &args[5];
    let id = &args[6];
    ensure!(
        !alias.is_empty()
            && alias.len() <= 64
            && alias
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
        "Invalid channel alias"
    );
    ensure!(
        root.is_absolute() && root.canonicalize()? == root,
        "Canonical absolute vault root required"
    );
    let uid = std::fs::metadata("/proc/self")?.uid();
    let meta = std::fs::symlink_metadata(root)?;
    ensure!(
        meta.is_dir() && meta.uid() == uid && meta.mode() & 0o777 == 0o700,
        "Vault root must be private and owned by current user"
    );
    let meta = std::fs::symlink_metadata(root.join(".secret_key"))?;
    ensure!(
        meta.is_file()
            && !meta.file_type().is_symlink()
            && meta.nlink() == 1
            && meta.uid() == uid
            && meta.mode() & 0o777 == 0o600,
        "Existing private vault key required"
    );
    let cipher: Arc<dyn InboxCipher> = Arc::new(VaultInboxCipher {
        store: zeroclaw_config::secrets::SecretStore::new(root, true),
        alias: alias.clone(),
    });
    cipher.check_existing()?;
    let mut owner = PrivateStore::open_existing(state)?;
    let db = owner.connection();
    // Inspection must not silently initialize an empty or legacy-only inbox.
    let tables: i64=db.query_row("SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('native_omemo_inbox','native_omemo_ingress_releases')",[],|r|r.get(0))?;
    ensure!(tables == 2, "Existing native ingress tables required");
    if operation == "inspect-release" {
        let receipt = inbox::inspect_release(db, &cipher, account, peer, id)?;
        return Ok(json!({"operation":operation,"release":receipt}));
    }
    if operation == "inspect" {
        let record = inbox::inspect(db, &cipher, account, peer, id)?;
        let phase = match record.phase {
            Phase::Pending => "pending",
            Phase::Admitted => "admitted",
            Phase::Returned => "returned",
        };
        return Ok(
            json!({"operation":operation,"phase":phase,"device":record.device,"fingerprint":hex::encode(record.pin),"runtime":record.runtime,"control":record.control,"review_token":hex::encode(record.review_token)}),
        );
    }
    let token: [u8; 32] = hex::decode(&args[7])?.try_into().map_err(|_| {
        ::zeroclaw_log::record!(
            ERROR,
            ::zeroclaw_log::Event::new(module_path!(), ::zeroclaw_log::Action::Fail),
            "Invalid review token length"
        );
        anyhow::Error::msg("Invalid review token length")
    })?;
    let operator = format!("uid:{uid}");
    inbox::release_reviewed_returned(
        db,
        &cipher,
        account,
        peer,
        id,
        inbox::ReleaseReview {
            expected_token: token,
            operator: &operator,
            evidence: &args[8],
        },
    )?;
    Ok(json!({"operation":operation,"released":true,"replayed":false}))
}
