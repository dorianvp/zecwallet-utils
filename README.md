# zecwallet-utils

A small Rust workspace with utilities for working with **ZecWallet Lite** wallet files
(`zecwallet-lite.dat`).

This repo currently contains:

- [`zecwallet-parser`](./parser) – library crate to parse ZecWallet Lite wallet files.
- [`zecwallet-dump`](./zecwallet-dump) – CLI tool to print a human-readable summary of a wallet file.

> ⚠️ **Security note**  
> ZecWallet Lite wallet files may contain private keys, seeds and other sensitive data.  
> Treat them carefully and don’t share them or example output unless you know what you’re doing.

---

## Crates

### `zecwallet-parser` (library)

A Rust library that decodes a `zecwallet-light-wallet.dat` file into a typed `ZwlWallet` struct.

---

### `zecwallet-dump` (CLI)

A small command-line tool to get a quick overview of a ZecWallet Lite wallet file.

Useful for:

- Sanity-checking a wallet backup.
- Debugging parsing issues.
- Quickly inspecting contents without running a full wallet client.
