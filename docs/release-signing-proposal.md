# Offline Release signing proposal

Status: proposal only. No signing dependency, embedded key, signature requirement or Release workflow change is implemented in 0.3.0.

## Recommendation

Use Minisign to sign a small release index, and embed its public key in the App and update helper. Keep the encrypted private key on an offline signing computer, with a separate encrypted offline backup. Never upload it to GitHub, a build service, a cloud-synced directory or GitHub Secrets.

Minisign supports detached signatures, password-protected secret keys and prehashed signatures. Use the current prehashed format and verify the signed comment as well as the payload. [Minisign documentation](https://jedisct1.github.io/minisign/)

The project's `minisign-verify` crate is a candidate for Rust integration. It offers verification without external dependencies, including streaming support. Review and pin an exact version and its license before implementation; do not write a new signature algorithm. [Rust verifier](https://github.com/jedisct1/rust-minisign-verify)

## What is signed

Add `release-index.json` and `release-index.json.minisig` to each Release. Sign the exact UTF-8 bytes; verification must precede parsing and must not reserialize the input.

The index should bind:

- A schema version and the `endNone/ClickPulse` repository identity.
- Release version, monotonic release sequence and exact source revision.
- Each supported OS and architecture, with its complete artifact names.
- Artifact byte sizes and SHA-256 hashes, including the existing package metadata.

The signed index authenticates archive hashes. Existing archive checks, payload allowlists, per-file hashes and safe extraction remain necessary. Unsigned GitHub metadata can locate an index but cannot establish its authenticity. Do not derive trust from a downloaded public key or a signature's key ID.

## Release workflow

1. Build all packages from the reviewed revision and assemble the index in an unpublished candidate.
2. Transfer the index and candidate packages to the offline computer using removable media.
3. Verify package hashes, target names and revision against the reviewed release record before signing.
4. Sign the index offline; return only the signature and public key fingerprint.
5. Verify the complete candidate with the embedded public key, then publish under the maintainer's account.

The signing computer must not execute downloaded packages or repository-provided signing scripts. Inspect the index as data. A signature proves maintainer approval of those bytes; it cannot establish that a compromised build produced trustworthy software. Build review remains separate.

## App and helper integration

In `src/updater/mod.rs`, fetch bounded metadata, verify the index, then validate its schema, repository, version and target. Require a newer version and a sequence above the highest previously accepted release. Reject missing, malformed, unknown-key or invalid signatures. Never fall back to unsigned installation after an error.

During staging, verify the archive against its authenticated hash before extracting. Persist the verified index and signature with the staged update. In `src/updater/install.rs`, the already-trusted helper must independently verify both again, then recheck the staged payload immediately before replacement. A plan file or an unsigned stored hash must not bypass verification. Preserve the existing backup and rollback behavior.

Keep the currently installed App working after any verification failure. Show a localized explanation through `ui/locales.json`. Scoop installations continue to use Scoop's update command.

## Bootstrap and key changes

A first version with an embedded key needs a trusted installation path; older Apps cannot retroactively authenticate their bootstrap update. Offer a manual installation with an independently published public-key fingerprint. State that limitation explicitly.

Rotate keys through a release verified by the old key that embeds the replacement key. Keep an overlap period for users who skip versions. Lost keys or compromised keys require a separately authenticated recovery release; never accept a new network-supplied key solely because GitHub serves it. Document the recovery route before rollout.

Version and sequence checks prevent downgrade after an accepted update. Without trusted freshness metadata, they do not prove that the network is showing the newest release. An expiry policy needs a separate design to avoid blocking infrequent releases or offline users.

Minisign does not replace Apple notarization or Windows publisher signing; it does not remove OS first-launch warnings.

## Acceptance before implementation ships

Test altered index bytes, altered archives, missing signatures, wrong keys, modified signed comments, wrong targets, replayed versions and oversized input. Exercise all supported platforms, helper restarts, tampering after staging, rollback, key rotation, skipped versions and the legacy bootstrap route. No release should claim signed-update protection until these checks pass.
