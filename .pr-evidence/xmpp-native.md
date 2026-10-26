# Native XMPP and legacy OMEMO validation

The native implementation replaces the historical compatibility worker.
The other XMPP evidence files retain their original revision and scope; their
WebUI screenshots remain configuration evidence, not native cryptographic proof.

## Reproducible boundary

The development-only driver in
`crates/zeroclaw-channels/tests/xmpp_native/managed_server.py` starts a disposable
loopback Prosody, generates synthetic accounts and a CA-signed server certificate,
and invokes the actual Rust `XmppChannel` with the shared agent dispatcher.
The Python reference peer is an optional development dependency, never part of
the channel runtime or release image. No production account, identity, database,
password or raw fixture report is committed.

On Linux with Rust 1.98.0, all four cases passed on 2026-10-26:

| Transport | Session | Observed result |
| --- | --- | --- |
| Direct TLS | Existing compatibility state | Encrypted shared-runtime reply decrypted |
| Direct TLS | Fresh prekey contact | Encrypted shared-runtime reply decrypted |
| Mandatory STARTTLS | Existing compatibility state | Encrypted shared-runtime reply decrypted |
| Mandatory STARTTLS | Fresh prekey contact | Encrypted shared-runtime reply decrypted |

The driver ended with:

```text
ACTUAL_CHANNEL_SHARED_RUNTIME_ENCRYPTED_SOCKET_ACK_REVOCATION_AND_CANCELLATION_PASSED
```

Each case also checked live allowlist/fingerprint revocation, exactly-once
admission of a duplicate stanza, preserved identity and other-device labels,
100 published one-time prekeys with non-reused IDs, listener cancellation and
reopen, encrypted ingress, canonical runtime linkage, rollback under injected
storage faults, pending recovery before admission, and no automatic replay of
admitted work after process exit. Offline administrative behavior is exercised
through child test executables; this does not claim a shipped administrator CLI.

Two shared-ingress regressions separately confirm that admission precedes hooks
and agent effects, including unowned approval routes, and that a hook rewrite
retains the original channel receipt.
The regular XMPP unit filter passed 60 tests, with four opt-in/child probes ignored
in that invocation; the network driver explicitly runs its ignored entrypoint.
Strict feature-scoped Clippy and formatting passed. Dependency policy reported
`advisories ok, bans ok, licenses ok, sources ok`; unmatched entries in the
existing policy were explained by the selected feature graph and crypto upgrade.
No advisory or license exemption was added.

## Limits

This verifies native legacy `eu.siacs.conversations.axolotl`, not OMEMO v2.
OMEMO private-state ownership currently requires Linux. Fresh outbound sessions
require the peer's first authenticated prekey message. Signed-prekey rotation,
copies to other bot devices, MUC, attachments and MAM are outside this slice.
The reference peer and local mock provider do not establish interoperability
with every client, actual remote-model/tool effects, production authentication,
or an audit of the cryptographic protocol implementation.

A socket acknowledgement does not prove delivery, decryption or reading.
Returned runtime processing does not prove tool success or persisted history.
Uncertain admitted input and reserved output require operator reconciliation.
Review migration and rollback using confidential state copies before deployment.
