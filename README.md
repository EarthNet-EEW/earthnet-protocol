> 🌎 Part of **[EarthNet](https://github.com/devjamez/earthnet)** — open-source, decentralized earthquake early warning for Latin America.

# earthnet-protocol

The spine of [EarthNet](https://github.com/develone/earthnet): the signed seismic
event wire format for the open, decentralized earthquake **early-warning** network.

Two messages, two trust levels:

| Message | Meaning | Trust |
|---------|---------|-------|
| `Observation` | Raw signed pick from a sensor (phone or official station) | Low — phones need consensus ≥ N |
| `ConfirmedEvent` | Post-fusion event that **triggers the alarm** | High |

Both are Protobuf messages signed with **Ed25519** over a deterministic payload
(`domain_tag || encode(message with signature = empty)`). The signing scheme is
normative — see [`PROTOCOL-v0.1-DRAFT.md`](../earthnet/PROTOCOL-v0.1-DRAFT.md).

## Build

No external `protoc` needed — the build uses the pure-Rust [`protox`](https://crates.io/crates/protox)
compiler.

```sh
cargo build
cargo test
```

## Status

🟡 v0.1 draft. Wire format may change (any change bumps `protocol_version` pre-1.0).

## License

Apache-2.0.
