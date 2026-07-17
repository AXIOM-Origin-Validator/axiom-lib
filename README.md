# axiom-lib

AXIOM general-purpose libraries: denomination math, rate limiting, the wire-visible error taxonomy.

Part of the AXIOM protocol family — specifications live in
[axiom-docs](https://github.com/AXIOM-Origin-Validator/axiom-docs), research in
[axiom-papers](https://github.com/AXIOM-Origin-Validator/axiom-papers), binaries in
[axiom-dist](https://github.com/AXIOM-Origin-Validator/axiom-dist).

## Contents

`denomination/` (currency-unit math) · `rate-limit/` (generic limiter) · `axiom-errors/` (the wire-visible error taxonomy — see the Yellow Paper Errors companion).

## Protocol pin

This repo builds standalone: its protocol dependencies are git-pinned to
[axiom-core](https://github.com/AXIOM-Origin-Validator/axiom-core) tag `core-b77fd28a`
(and [axiom-lib](https://github.com/AXIOM-Origin-Validator/axiom-lib) tag `lib-v3.3.0`).
Upgrading the pin is a deliberate act — the protocol evolves slowly by design.

## Releases

This repository receives one snapshot commit per AXIOM release, exported from
the project's working tree (3.3.0 at export). Its git log is the release
history. License: GPL-3.0.

> AXIOM is pre-mainnet software. Do not use it to custody real value.
