# Credential UI evidence

Chrome headless, 1280 and 390 CSS pixels. The harness renders the actual HTTP Credentials component, its translated labels, shared button and CSS from each PR commit. It uses a mock catalog with synthetic values and displays no secrets. It does not exercise a live gateway and does not capture the inherited plugin TLS or Settings tabs.

- PR 12408, 391cce0: username/password and API token empty, disabled, created denied, granted, revoked, replace and delete confirmation.
- PR 12490, cab1b6b: Token grant/revoke, create selector, preserved Token on replace, and encryption-disabled.
