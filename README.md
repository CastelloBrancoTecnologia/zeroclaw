# Full-application Token evidence for #12490

Source: `08f0f83cc5f3dea508bd5431e7c7f88e8bb0cee7`. Captured 2026-11-27.

Unmodified full App/router/Config through Vite at the stated source head.

Real config_admin_router, ConfigRouteAuth, paired-browser authentication and encrypted temporary-file persistence at stated source head; test-only loopback fixture.patch adds bootstrap health/auth-methods responses and synthetic plugin state. No production handler changes.

All destinations, aliases and values are synthetic. Browser captures are unedited. Mobile replacement/create forms were scrolled to expose the complete form. This is a local authenticated configuration exercise, not production deployment or outbound HTTP authentication. Nothing contacted api.example.com.

## Captures

| State | Desktop 1280×900 | Narrow 390×844 |
|---|---|---|
| token-create | [View](desktop-token-create.jpg) | [View](mobile-token-create.jpg) |
| token-created-denied | [View](desktop-token-created-denied.jpg) | [View](mobile-token-created-denied.jpg) |
| token-empty | [View](desktop-token-empty.jpg) | [View](mobile-token-empty.jpg) |
| token-encryption-disabled | [View](desktop-token-encryption-disabled.jpg) | [View](mobile-token-encryption-disabled.jpg) |
| token-granted | [View](desktop-token-granted.jpg) | [View](mobile-token-granted.jpg) |
| token-replace-preserved | [View](desktop-token-replace-preserved.jpg) | [View](mobile-token-replace-preserved.jpg) |
| token-revoked | [View](desktop-token-revoked.jpg) | [View](mobile-token-revoked.jpg) |

The create form selects Authorization: Token; the real handler persists it. Grant/revoke controls reflect real catalog updates. The replacement form retains Token and leaves the stored secret blank. Replacement was submitted successfully. With encryption disabled in the disposable fixture, mutation controls are disabled; encryption was restored afterward.

## Regression tests

Commit 08f0f83cc5f3dea508bd5431e7c7f88e8bb0cee7 also carries the two route regressions omitted from this child, specialized to Token. They check default-denied creation, same-origin-only grant retention, the returned token header, and unchanged persistence after generic gateway write attempts.

```text
running 2 tests
test principal_gate::tests::scoped_http_catalog_refuses_generic_gateway_writes ... ok
test principal_gate::tests::scoped_http_rotation_preserves_only_same_origin_grants ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 947 filtered out; finished in 0.30s
```

Complete principal gate suite: **34 passed, 0 failed, 1 ignored**. The ignored test is the manual evidence server; it was run separately for these captures. See principal-tests.log.

## Reproduce

At the stated head, apply fixture.patch locally. Build with `cargo test --locked -p zeroclaw-gateway --features plugins-wasm --lib full_ui_evidence_server --no-run`. Run the resulting test binary with `principal_gate::tests::full_ui_evidence_server --ignored --nocapture`; it binds 127.0.0.1:42617 and prints a fresh pairing code. Run the unchanged frontend with `npm run dev -- --host 127.0.0.1 --port 5173 --strictPort`. Open `/config/secrets?tab=api-key-credentials` and pair through the normal form. Exercise the tabs and restore the disposable encryption setting after the disabled-state capture. Stop both servers and revert the local fixture patch. Do not commit it to the feature branch.

The temporary fixture was removed from the checkout after compilation; it was not pushed with the feature. The fixture supplies non-configuration daemon/model doubles, not a full daemon startup claim.

## Landing holds

#12408 and #12563 remain prerequisites. Rebuild the stack after they merge and obtain human re-review. This evidence does not grant merge approval.
