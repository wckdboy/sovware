# sovware

Tools for humans and AI agents to build things that are checked, not trusted.

**Status:** the kernel runs. One domain is draft. Nothing here is admitted for
fabrication. That is deliberate — see [`MANIFESTO.md`](MANIFESTO.md).

```sh
make demo
```

```
▪ mechanical/lug-bracket @1 · mechanical · cadquery.part
  ✓ sov/source            0.05s  score 1.000
  ✓ sov/units             0.03s  score 1.000
  ✓ sov/claims            0.04s  score 1.000
  ✗ mech/yield-margin     0.04s  score 0.455
     └ MECH-YIELD-002 bearing: 227.3 MPa of 414.0 MPa allowable, margin 0.91 < 1.00 at SF 2.0
✗ gate Fail · score 0.455 · 51.8g · 0.12s · failed: mech/yield-margin
  run_6360743c8f50 → ledger

...

▪ improvement v1 → v2
  ✓ gate      parent Fail · child Pass
  ✓ score     0.455 → 0.622
  ✓ cost      mass 51.8g → 40.5g
  improvement

▪ the cheaper mutation v1 → v3
  ✗ gate      parent Fail · child Fail
  ✗ score     0.455 → 0.285
  ✓ cost      mass 51.8g → 32.4g
  change
  not an improvement. recorded either way (SPEC R11).
```

A cheaper design that fails the gate is a change, not an improvement. The tool says so,
and keeps the record. That distinction is the whole project.

## The loop

```
program ──▶ verifiers ──▶ gate ──▶ ledger
   ▲                                  │
   └────────── mutation ◀─────────────┘
```

Four objects, no more:

| Object | What it is | Schema |
| --- | --- | --- |
| **program** | a typed, re-derivable artifact: params, units, claims, source hash | [`schema/program`](schema/program.schema.json) |
| **verifier** | an out-of-process executable that returns a verdict, a score and a cost | [`schema/verifier`](schema/verifier.schema.json) |
| **gate** | the domain's rule for turning scores into pass, fail, or refusal | [`schema/domain`](schema/domain.schema.json) |
| **ledger** | append-only, hash-chained evidence of every run | [`schema/run`](schema/run.schema.json) |

An improvement is defined, not asserted: gate pass, score not worse on a held-out set,
cost not higher. Everything else is a change, and changes are recorded too.

## Quickstart

Rust 1.91+ and Python 3.10+ (the reference verifiers are stdlib only).

```sh
cargo build
./target/debug/sov verify examples/bracket/lug-bracket.v2.json --actor human:you
./target/debug/sov ledger verify
./target/debug/sov compare run_a run_b
make check     # fmt, clippy, tests, schema parse, the full loop offline
```

The kernel does not need a model, a key, or the network. That is a requirement, not an
accident (`MANIFESTO` §12).

## What is in here

```
MANIFESTO.md          the doctrine. outranks the spec, which outranks the code.
docs/SPEC.md          SOV-SPEC-1. normative, with requirement IDs and tests.
docs/FORKS.md         build/buy/wrap/fork decisions and the license boundary.
schema/               the six wire objects, including the eval corpus.
crates/sov-core/      types, hashing, verifier execution, gate, ledger.
crates/sov-cli/       `sov verify | compare | ledger | hash`.
verifiers/            four reference verifiers, all deterministic, all offline.
domains/mechanical.json   the first gate, with its non-goals.
examples/bracket/     one part, three mutations, one improvement.
```

## Adding a domain

A domain is admitted by its verifiers, not by a vendor list (`R5`, `SPEC §8`):

1. write `domains/<id>.json` with artifacts, gate, budget and non-goals;
2. write at least two exact verifiers and one numeric or simulation verifier;
3. collect 30 programs, hold out 10, and include five known-bad ones;
4. prove it offline, then publish both the passes and the failures (`R12`).

The gap between "the demo works" and "the gate is honest" is the corpus. Start with the
corpus.

## The rules that matter most

- **R2** every verifier declares `exact`, `numeric` or `statistical`. Statistical never
  gates.
- **R8** an errored verifier is a refusal, not a pass.
- **R9** nothing moves matter without a human signature. The kernel cannot call a printer,
  and neither can an agent.
- **R11** improvement means gate pass **and** no worse on held-out **and** no more
  expensive. Otherwise we say "change".

Full list in [`docs/SPEC.md`](docs/SPEC.md).

## License

Apache-2.0. The kernel is permissive on purpose: the value is in the verifiers and the
ledger, not in a lock. Tools we call keep their own licenses behind a process boundary
(`docs/FORKS.md`).

WCKD lab · [sovware.io](https://sovware.io)
