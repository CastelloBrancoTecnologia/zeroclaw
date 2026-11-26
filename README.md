# Full-application evidence for #12408

Source revision: `284104bdbaf3b85c386d90285c7ba54a895194a0`. Captured 2026-11-26.

Unmodified full App/BrowserRouter/Config frontend through Vite; no component mounting harness or API mocks.

Real config_admin_router, real ConfigRouteAuth, real paired-browser authentication, real encrypted temporary-file persistence. Test-only fixture.patch adds loopback server/bootstrap health/auth-methods routes and synthetic plugin manifest; no production handler changes. Other daemon/model subsystems are test_state doubles; no external HTTP credential dispatch or TLS connection acceptance claimed.

Desktop: 1280 × 900. Narrow: 390 × 844. All values and destinations are synthetic. No live credentials, production hostname, account identity, or private transcript is present. Images are original browser captures; no image compositing or redaction edits were applied. Mobile replacement forms were scrolled to show the inputs and action buttons.

The TLS `unset` view is an admitted plugin secret slot without a stored value; this is distinct from a plugin catalog with no configured slots. TLS has no HTTP grant toggle. No no-plugin-catalog capture is claimed.

## Capture index

| State | Desktop | Narrow |
|---|---|---|
| api-created-denied | [View](desktop-api-created-denied.jpg) | [View](mobile-api-created-denied.jpg) |
| api-delete-confirmation | [View](desktop-api-delete-confirmation.jpg) | [View](mobile-api-delete-confirmation.jpg) |
| api-empty | [View](desktop-api-empty.jpg) | [View](mobile-api-empty.jpg) |
| api-encryption-disabled | [View](desktop-api-encryption-disabled.jpg) | [View](mobile-api-encryption-disabled.jpg) |
| api-granted | [View](desktop-api-granted.jpg) | [View](mobile-api-granted.jpg) |
| api-replace | [View](desktop-api-replace.jpg) | [View](mobile-api-replace.jpg) |
| api-revoked | [View](desktop-api-revoked.jpg) | [View](mobile-api-revoked.jpg) |
| basic-created-denied | [View](desktop-basic-created-denied.jpg) | [View](mobile-basic-created-denied.jpg) |
| basic-delete-confirmation | [View](desktop-basic-delete-confirmation.jpg) | [View](mobile-basic-delete-confirmation.jpg) |
| basic-empty | [View](desktop-basic-empty.jpg) | [View](mobile-basic-empty.jpg) |
| basic-encryption-disabled | [View](desktop-basic-encryption-disabled.jpg) | [View](mobile-basic-encryption-disabled.jpg) |
| basic-granted | [View](desktop-basic-granted.jpg) | [View](mobile-basic-granted.jpg) |
| basic-replace | [View](desktop-basic-replace.jpg) | [View](mobile-basic-replace.jpg) |
| basic-revoked | [View](desktop-basic-revoked.jpg) | [View](mobile-basic-revoked.jpg) |
| settings-encryption-disabled | [View](desktop-settings-encryption-disabled.jpg) | [View](mobile-settings-encryption-disabled.jpg) |
| settings | [View](desktop-settings.jpg) | [View](mobile-settings.jpg) |
| tls-configured | [View](desktop-tls-configured.jpg) | [View](mobile-tls-configured.jpg) |
| tls-delete-confirmation | [View](desktop-tls-delete-confirmation.jpg) | [View](mobile-tls-delete-confirmation.jpg) |
| tls-encryption-disabled | [View](desktop-tls-encryption-disabled.jpg) | [View](mobile-tls-encryption-disabled.jpg) |
| tls-replace | [View](desktop-tls-replace.jpg) | [View](mobile-tls-replace.jpg) |
| tls-unset | [View](desktop-tls-unset.jpg) | [View](mobile-tls-unset.jpg) |

## Reproduction

Use source revision above. Apply `fixture.patch` locally (test-only; do not add it to the feature PR). Build with `cargo test --locked -p zeroclaw-gateway --features plugins-wasm --lib full_ui_evidence_server --no-run`. Run the resulting test binary with `principal_gate::tests::full_ui_evidence_server --ignored --nocapture`. It binds only to 127.0.0.1:42617 and prints a fresh pairing code. Start the unchanged web frontend with `npm run dev -- --host 127.0.0.1 --port 5173 --strictPort`, open `/config/secrets`, and pair through the real pairing form. Use the four tabs and their forms. For disabled states, change and save Encrypt only in this disposable fixture; restore it afterward. Stop the fixture and revert the test-only patch after capture.

The gateway config, key and plugin manifest live in a temporary directory. No credential requests are sent to example.com; only local configuration is exercised. No actual plugin Wasm is executed. Bootstrap health and auth-methods responses are fixture routes, not proof of full-daemon startup.

## Tests repeated on this source

```text
running 2 tests
test principal_gate::tests::scoped_http_catalog_refuses_generic_gateway_writes ... ok
test principal_gate::tests::scoped_http_rotation_preserves_only_same_origin_grants ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 947 filtered out; finished in 0.30s
```

The first shared-target compilation reused an older config artifact and failed with missing scoped HTTP types. It was not counted as validation. Rebuilding the exact-source config and dependent crates succeeded, followed by the above passing actual-router tests. The temporary helper has been removed from the feature checkout.

## Remaining landing prerequisites

#12402 and #12563 still need to merge; rebuild this stack on the merged parent and obtain human re-review. These captures do not remove a maintainer review or label.
