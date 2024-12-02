# Core BPF Migratoooor

Can run "stub tests", which simulate a migration on a test validator, or
"conformance tests", which clone an ELF from a deployed buffer and run it
against the builtin within Firedancer's conformance tooling.

Fixtures tests are similar to conformance but without comparing to a builtin.
Instead, fixtures are just run against the ELF.

Stub tests:

```
cargo run --release --bin cbm -- stub <program>
```

Fixtures tests:

```
cargo run --release --bin cbm -- fixtures <program> --cluster mainnet-beta

Conformance tests:

```
cargo run --release --bin cbm -- conformance <program> --cluster mainnet-beta
```

Supported programs:

- `address-lookup-table`
- `config`
- `feature-gate`

## Notes

Depends on Joe C's fork of Agave, solely for one silly change to the test
validator's genesis configuration. See
https://github.com/anza-xyz/agave/pull/3766.
Fork is based on Agave v2.0.7, since that's the minimum required Agave version
to activate a Core BPF migration feature.
