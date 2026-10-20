# working in this repo

Rules for agents and humans. They are short because the manifesto is long.

## before you change anything

Read, in this order: `MANIFESTO.md`, `docs/SPEC.md`, `docs/FORKS.md`. The manifesto
outranks the spec, which outranks the code. If a change makes one of them false, change
the document first, in the same PR.

## the invariants

1. **`make check` is the merge condition.** fmt, clippy with `-D warnings`, tests, schema
   parse, and the offline loop. If it fails, the change is not done.
2. **No new dependency without a `docs/FORKS.md` entry.** Say what it is, its license, and
   whether it is linked, called, or served.
3. **No verifier without a declared kind.** `exact`, `numeric`, or `statistical`. A
   statistical verifier does not gate. Never default it.
4. **No claim without a source.** If you add a material constant, a standard reference, or
   an assumption, it goes in `claims` with a resolvable `ref`.
5. **Never set `fabrication.ready: true`.** That is a human decision with a signature
   (`R9`). An agent that does this has broken the system on purpose.
6. **The ledger is append-only.** Do not edit, reorder, or regenerate a committed
   `runs.jsonl`. Append.
7. **Do not tune on the held-out set.** Do not look at `holdout/` when proposing a
   mutation. If you did, say so and the run is void.
8. **A failing example stays in the repo.** `lug-bracket.v3.json` exists to prove the gate
   rejects cheap wrong answers. Do not delete it to make the demo look better.

## writing style

Maintainer, not marketer. Short declarative sentences, concrete nouns, numbers where they
exist. State the limit as plainly as the feature. No exclamation marks, no superlatives,
no claims without a link or a run id.

## adding a verifier

```
verifiers/<namespace>-<name>/
  verifier.json   schema, id, kind, gates, applies_to, invoke, license, checks
  main.py         reads a program on stdin, writes one result on stdout
```

- stdlib only in the reference verifiers. A heavy dependency is fine, but it must be
  justified in `docs/FORKS.md` and pinned.
- deterministic given the same bytes and environment. If it is not, declare `statistical`.
- exit code is advisory; the verdict is authoritative.
- one JSON object on stdout. Nothing else.
- fail closed: unknown unit, unknown material, missing parameter → `fail` or `error`, never
  `pass` by omission.

Then add it to a domain's `gate.require`, and prove it rejects a known-bad program before
you prove it accepts a good one.

## commits

One idea per commit. Body explains the failure it prevents, not the files it touches.
Reference a requirement id (`R2`, `R8`) when the change is normative.
