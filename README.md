# Core BPF Migratoooor

```
./run.sh <program>
```

Supported programs:

- `address-lookup-table`
- `config`
- `feature-gate`

## Notes

Depends on Joe C's fork of Agave, solely for one silly change to the test
validator's genesis configuration. See
https://github.com/buffalojoec/solana/tree/joec-test-validator-genesis.
Fork is based on Agave v2.0.7, since that's the minimum required Agave version
to activate a Core BPF migration feature.
