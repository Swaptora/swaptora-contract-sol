# Swaptora Escrow Program

An Anchor program for addressed, atomic swaps on Solana. A maker deposits the
offered assets into a program-derived vault; only the named taker can accept the
offer before it expires. The program never holds user keys and has no relayer or
backend custody component.

> **Status: pre-deployment / unaudited.** This repository is suitable for local
> development and review. Do not use it with real assets until an audited,
> verified build has been deployed and the release information below is filled in.

## At a glance

| Property | Value |
| --- | --- |
| Framework | Anchor 1.1.2 |
| Program language | Rust |
| Program ID compiled into this source | `CUziHakzRiAYkYE5kz5Sb3DzyWor4p51QRpQ99HvKib8` |
| Deployment status | Not verified on devnet or mainnet-beta |
| Escrow lifetime | 604,800 seconds (7 days) |
| Assets per side | Up to 8 |
| Program test harness | Mollusk 0.14 against the compiled sBPF ELF |

The program ID above is a build-time address, **not** evidence of an on-chain
deployment. Do not send SOL or tokens to it. A released deployment must publish
the cluster, deployed program address, transaction, source revision, IDL, binary
hash, upgrade-authority status, and verified-build record.

## Supported assets

| Asset | Supported in v1 | Validation performed |
| --- | --- | --- |
| Native SOL | Yes | Exact lamport amount is escrowed in the vault PDA. |
| Classic SPL Token | Yes | The mint and token accounts must belong to the classic SPL Token Program; source and destination accounts are canonical ATAs. Frozen accounts and mints with a freeze authority are rejected. |
| Metaplex `NonFungible` NFT | Yes | Classic SPL mint; supply `1`; decimals `0`; amount `1`; canonical Metaplex Token Metadata PDA; `token_standard = NonFungible`. A collection is optional and is not used as an admission rule. |
| Token-2022 | No | Rejected in v1, including all Token-2022 extensions. |
| Programmable NFT, compressed NFT, Metaplex Core | No | Not implemented in v1. |

The contract verifies technical asset identity, not market value, creator
identity, intellectual-property rights, or collection authenticity. Integrators
must display the exact mint address, token-program variant, and collection state
to users before they sign a transaction.

## Protocol flow

1. The one-time release initializer calls `initialize_config` with the immutable
   fee receiver and platform fee.
2. The maker calls `create_offer`, pays the platform fee, and deposits its side
   of the trade into the per-offer vault.
3. Before the deadline, the named taker calls `accept_offer`. Both sides transfer
   atomically; token vault ATAs are closed and their rent returns to the maker.
4. Before expiry, the maker may call `cancel_offer` to recover the escrow.
5. After expiry, anyone may call `claim_expired_offer`; assets still return only
   to the maker.

Failed instructions roll back atomically. Any unsolicited SOL or classic SPL
tokens sent to a vault are returned to the maker when the offer is completed,
cancelled, or reclaimed.

## Instructions

| Instruction | Caller | Purpose |
| --- | --- | --- |
| `initialize_config` | Release initializer, once | Creates the immutable `Config` PDA with the fee receiver and fee. |
| `create_offer` | Maker | Creates an active addressed offer and escrows the maker assets. |
| `accept_offer` | Named taker | Atomically exchanges the two sides before expiry. |
| `cancel_offer` | Maker | Refunds an active offer before expiry. |
| `claim_expired_offer` | Any signer | Refunds an expired active offer to its maker. |

Detailed ordering and mutability of dynamic accounts is in
[the account schema](docs/ACCOUNT_SCHEMA.md). Clients must pass exactly that
schema; missing, surplus, duplicate, or substituted remaining accounts are
rejected.

## Accounts and PDAs

```text
Config:            ["config"]
Offer:             ["offer", maker, nonce.to_le_bytes()]
Vault:             ["vault", offer]
Vault token ATA:   ATA(owner = Vault PDA, mint, classic SPL Token Program)
```

`Offer` stores the participants, terms, fee snapshot, timestamps, status, and
PDA bumps. The initial status is `Active`; there is no on-chain draft state.

## Local development

### Prerequisites

- Rust 1.89 or newer
- Solana CLI 3.1.10 with platform-tools v1.52
- Anchor CLI 1.1.2

`Cargo.lock` is committed intentionally to keep host and SBPF dependencies
reproducible.

### Build, test, and generate the interface

```bash
NO_DNA=1 cargo fmt --all -- --check
NO_DNA=1 cargo build-sbf --tools-version v1.52
NO_DNA=1 anchor run mollusk
NO_DNA=1 cargo clippy --all-targets -- -D warnings
NO_DNA=1 anchor idl build \
  -o idl/swaptora_contract_sol.json \
  -t types/swaptora_contract_sol.ts
```

The Mollusk suite runs the compiled sBPF artifact, not a host-only substitute.
It covers SOL, classic SPL, NFT and mixed-asset swaps; atomic rollback; wrong
taker and replay attempts; cancel and permissionless expiry reclaim; account
substitution; token-account freezing; Token-2022 rejection; an NFT with an
unverified collection; and an NFT with no collection.

Generated client artifacts are committed:

- [Anchor IDL](idl/swaptora_contract_sol.json)
- [TypeScript IDL type](types/swaptora_contract_sol.ts)

## Security model and limitations

- The platform fee is charged only after a successful `create_offer` and is not
  refundable.
- The configured fee receiver and fee are immutable after initialization.
- v1 intentionally has no pause, fee-update, asset-restriction,
  emergency-withdraw, or administrative asset-transfer instruction.
- The current `RELEASE_INITIALIZER` is a local-test placeholder. It must be
  replaced with a dedicated public key for a release; never commit its private
  key.
- This code has not been independently audited. An audit, devnet integration
  testing, reproducible build verification, and a review of upgrade authority
  are required before production use.

See the complete [release checklist](docs/RELEASE.md). For Solana deployment and
verified-build guidance, consult the official [Solana verified builds
documentation](https://solana.com/docs/programs/verified-builds).

## Repository conventions

- Rust source: [`src/`](src)
- SBPF/Mollusk tests: [`tests/mollusk.rs`](tests/mollusk.rs)
- Account ABI: [`docs/ACCOUNT_SCHEMA.md`](docs/ACCOUNT_SCHEMA.md)
- Generated public interfaces: [`idl/`](idl) and [`types/`](types)

Changes to account layouts, instruction arguments, error ordering, PDA seeds, or
asset-validation rules are compatibility-sensitive. Treat them as a protocol
version change: regenerate the IDL/types, update tests and documentation, and
deploy a new program ID unless a reviewed migration path exists.

## Contributing and security reports

Before opening this repository for external contributions, add `CONTRIBUTING.md`,
`SECURITY.md`, a `LICENSE` file, CI, and a monitored vulnerability-reporting
channel. Until then, do not disclose potential vulnerabilities in a public issue
and do not submit production deployment requests through this repository.

The crate currently declares the MIT license in [`Cargo.toml`](Cargo.toml). The
full MIT license text must be committed as `LICENSE` before the repository is
made public.
