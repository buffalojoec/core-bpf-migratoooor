# Core BPF Migration Test CLI

A CLI to test different aspects of upcoming migrations of Solana builtins to
Core BPF.

Supported programs:

- `address-lookup-table`
- `config`
- `feature-gate`

## Stub Testing

This test will spin up a test validator with a mock BPF program's ELF in place
of the source buffer account. It also assigns the feature accounts to another
custom program, which simply reassigns them to `Feature1111...`, effectively
activating them without a keypair.

The purpose of this test is to ensure the runtime mechanism for migrating the
program works as expected, and the program can be successfully invoked after
migration is completed.

```
cargo run --release --bin cbmt -- stub <program>
```

## Fixtures Testing

Using Firedancer's [solana-conformance](https://github.com/firedancer-io/solana-conformance)
tool, run a set of fixtures against the program, using an ELF cloned from a
buffer account in some higher cluster.

The purpose of this test is to ensure the deployed ELF is in fact the expected
BPF version of the program and can successfully process every fixture and
produce the expected result (effect).

```
cargo run --release --bin cbmt -- fixtures <program> --cluster mainnet-beta
```

Note you have the option to use Firedancer's fixtures or the fixtures generated
by [mollusk](https://github.com/buffalojoec/mollusk), stored in the program's
repository under `program/fuzz`. Simply provide the `--use-mollusk-fixtures`
option.

## Conformance Testing

Using Firedancer's [solana-conformance](https://github.com/firedancer-io/solana-conformance)
tool, run a set of fixtures against both the original builtin and the BPF
version of the program, using an ELF cloned from a buffer account in some
higher cluster.

The purpose of this test is similar to the above [Fixtures Testing](#fixtures-testing),
only it also tests for conformance against the original builtin. It could be
considered overkill, but it's available nonetheless.

```
cargo run --release --bin cbmt -- conformance <program> --cluster mainnet-beta
```

Note you have the option to use Firedancer's fixtures or the fixtures generated
by [mollusk](https://github.com/buffalojoec/mollusk), stored in the program's
repository under `program/fuzz`. Simply provide the `--use-mollusk-fixtures`
option.

## A Note on the Agave Fork Dependency

This harness temporarily depends on Joe C's fork of Agave in order to allow
feature accounts to be created and not overwritten/erased by the test
validator's startup process (genesis config).

For more information, see https://github.com/anza-xyz/agave/pull/3766.

The fork is also based on Agave v2.0.7, since that's the minimum required Agave
version to activate a Core BPF migration feature.
