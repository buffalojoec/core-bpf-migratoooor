# Core BPF Migratoooor

```
scripts/fetch_program.sh <program>
cargo run -- test <program>
```

Supported programs:

- `address-lookup-table`
- `config`
- `feature-gate`

## Notes

Solana versions can be tweaked in the workspace manifest, but the default is
set to `2.0.7`, since that's the minimum required Agave version to activate a
Core BPF migration feature.
