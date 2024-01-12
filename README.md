# Programify Feature Gate

TODO: Verifiably build this program.

This repository housees a no-op program to be used as a placeholder for
"programifying" the Feature Gate program at
`Feature111111111111111111111111111111111111`.

This no-op program will be deployed as an upgradeable BPF program at some
arbitrary address, and then, using a runtime feature gate, it will be moved in
place of the non-existent account at
`Feature111111111111111111111111111111111111`.

To facilitate testing, this repository has a simple CLI for sending
transactions to `Feature111111111111111111111111111111111111` and testing the
response recieved to ensure the migration was successful.

### To Run the Test

```
./start
```