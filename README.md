# stellar-trustline-demo

Rust implementation of "establish a trustline, then send an issued asset" on
Stellar, using:

- **stellar-base** (`0.7`) — builds and signs transactions offline.
- **stellar_sdk** (`0.1`) — talks to Horizon (loads accounts, submits transactions).

## Layout

- `establish_trustline()` — loads the account, builds a `ChangeTrust` operation,
  signs it, and submits it to Horizon.
- `wait_for_trustline()` — polls the account until the trustline shows up in its
  balances (or times out). This exists because a payment in a credit asset fails
  with `op_no_trust` if it's submitted before the trustline transaction has been
  confirmed on the ledger — see the walkthrough's "Why the Order Matters" section.
- `send_asset()` — reloads the (now-advanced) sequence number, builds a `Payment`
  operation for the credit asset, signs it, and submits it.
- `main()` — wires the three together: connect → identities → trust → wait → pay.

## Before running

1. `cargo build` — this pulls `stellar-base` and `stellar_sdk` from crates.io, so
   it needs network access (this sandbox doesn't have any, so I wasn't able to
   compile it here — build it locally to confirm).
2. Replace the placeholder `"G...RECIPIENT"` with a real testnet address.
3. A freshly generated keypair (`DalekKeyPair::random()`) has no XLM and doesn't
   exist on the ledger yet. Fund it first via Friendbot before running:
   ```bash
   curl "https://friendbot.stellar.org/?addr=<YOUR_G_ADDRESS>"
   ```
4. `wait_for_trustline()` reads `asset_code` / `asset_issuer` off each entry in
   `account.balances`. If those field names don't match your installed version of
   `stellar_sdk` exactly, check `cargo doc --open` for the `Balance` type and
   adjust — it's a convenience check, not part of the core trust/pay flow.

## Run

```bash
cargo run
```
