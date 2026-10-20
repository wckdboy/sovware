# sovware specification 1.0

**SOV-SPEC-1** · status: normative · supersedes `ASC-SYS-2026-V4.2`

This document defines the kernel: how an artifact is represented, how it is scored, how a
mutation is accepted, and what must be true before any of it is believed. It replaces a
document that named vendors. This one names requirements.

The key words MUST, MUST NOT, SHOULD, and MAY are used as in RFC 2119.

---

## 1. scope

The kernel is the smallest thing that can be shared by every Sovware domain. It has four
parts and no more:

| Part | Definition |
| --- | --- |
| **program** | a typed, versioned, re-derivable representation of an artifact |
| **verifier** | an out-of-process executable that scores a program |
| **gate** | the rule that turns scores into pass or fail for a domain |
| **ledger** | an append-only, hash-chained record of every run |

Everything else — CAD kernels, solvers, search, model routing, fabrication — is a domain
implementation or a host, and lives outside the kernel.

### 1.1 non-goals

This specification does not define:

- which model, vendor, solver, or language is used;
- how a domain computes its physics (that is the domain's verifier);
- user interface, agent prompt topology, or job scheduling (see `omnie-harness`);
- fabrication control. That is explicitly out of scope and gated by R9.

A conformance claim against this document says nothing about whether a part is safe to
build. It says the loop that judged it is honest.

---

## 2. definitions

**program** — a JSON object conforming to `schema/program.schema.json`. It MUST declare
`id`, `domain`, `kind`, `params`, and `source`, where `source.sha256` is the hash of the
bytes that generate the artifact.

**verifier** — an executable described by `schema/verifier.schema.json`. It reads a program
on stdin or from a path and writes one `verifier-result` object to stdout.

**verdict** — one of `pass`, `fail`, `error`, `skip`. `error` is not `fail`: an errored
verifier means the gate is unknown, and an unknown gate is a refusal, not a pass.

**score** — a number in `[0, 1]` for verifiers of kind `numeric` or `statistical`. Exact
verifiers return `1.0` or `0.0`. Score is monotone: higher is better, always.

**gate** — the domain's acceptance rule over a set of results (see §5).

**run** — one execution of the gate over one program, recorded in the ledger
(`schema/run.schema.json`).

**held-out set** — programs and expected verdicts that were fixed before the mutation was
proposed and are not visible to the mutating agent.

**improvement** — a mutation satisfying all three conditions in §7.

---

## 3. requirements

Each requirement is testable. The test column names the check that decides it, and `sov
verify` or CI runs it.

### R1 — the artifact is a program

Every artifact MUST be reproducible from `source` plus declared `params`, with no manual
step. A verifier MUST be able to reject an artifact whose hash does not match its source.

*Test:* rebuild from source in a clean environment; compare `source.sha256`; the artifact's
`reproducible` verifier passes.

### R2 — every verifier declares its kind

A verifier MUST declare `kind` ∈ {`exact`, `numeric`, `statistical`} and MUST declare
whether it gates. `statistical` verifiers MUST NOT gate. A verifier with no declared kind
MUST be rejected by the kernel, not defaulted.

*Test:* schema validation; kernel rejects an undeclared verifier with `SOV-VERIFIER-KIND`.

### R3 — no unsourced claims

Every entry in `claims` MUST carry a `source` with `kind` ∈ {`standard`, `paper`,
`measurement`, `test`, `derivation`} and a resolvable `ref`. A claim used to justify a
parameter MUST be referenced by that parameter.

*Test:* the `sov/claims` verifier fails on a claim with an empty or unresolvable ref.

### R4 — the agent mutates source, not results

The kernel MUST accept only programs. It MUST NOT accept a mesh, an image, a rendered
schematic, or a binary as the unit of mutation. Derived artifacts MAY be attached as
evidence and MUST be labelled `derived`.

*Test:* `program.kind` is drawn from a registered list; `sov verify` rejects an artifact
without `source`.

### R5 — domains are admitted by gate, not by vendor

A domain MUST be admitted by publishing: its artifact kinds, its gate composition, its
verifier set with versions, and its non-goals. No domain is admitted because a product
exists for it.

*Test:* `sov domains check <domain.json>` fails unless every verifier named in the gate
resolves and every artifact kind is declared.

### R6 — verifiers run out of process

Every verifier MUST be executed as a subprocess with an explicit timeout and a declared
input mode. The kernel MUST NOT link a verifier into its own address space. A verifier
MAY be any language, any license, any runtime.

*Test:* the reference kernel's only execution path is `Command`; a verifier that exceeds
`invoke.timeout_s` returns `error`.

### R7 — the ledger is append-only and chained

Every run MUST be appended to a ledger as one JSON object per line, and MUST carry
`prev` = the hash of the preceding run (or `null` for the first). The run `hash` MUST cover
everything except `hash` itself, canonically serialized.

*Test:* `sov ledger verify` replays a ledger and fails on a broken chain, a reordered line,
or an edited score.

### R8 — an errored verifier is a refusal

A gate MUST NOT pass when any gating verifier returned `error` or was not run. Missing
evidence is a fail, and the run MUST record why.

*Test:* delete a verifier from the set; the gate fails with `SOV-GATE-MISSING`.

### R9 — the human gate

A program MAY be marked `fabrication.ready`. The kernel MUST NOT transmit a program to a
machine, a vendor, or a printer. Any downstream consumer MUST require a ledger run with
`gate.result = pass`, a human actor signature, and a program hash that matches.

*Test:* `sov verify --require-signature` fails on a run whose actor is not `human:<id>`.

### R10 — cost is carried with the score

Every run MUST record wall time, and SHOULD record dollars, material, and mass where the
domain declares them. The gate MAY reject on a budget, but a rejected-for-cost run MUST
still be recorded.

*Test:* `sov verify --budget wall_s=60` fails and records the run.

### R11 — improvement is defined, not asserted

A change is an improvement only if §7 holds. Any published claim of improvement MUST link
to the two ledger runs (parent, child) and the held-out set hash.

*Test:* `sov compare <parent> <child> --set <set>` prints `improvement`, `change`, or
`regression`, and only `improvement` may be quoted as such; `sov eval` refuses a set whose
cases no longer match the published `holdout_hash`.

### R12 — the record is public

A conformance report MUST include failures and regressions, not only passes. A benchmark
used for tuning MUST be published with its hash and MUST NOT be used as the held-out set.

*Test:* release checklist; `sov report` refuses to render a report that omits failed runs.

---

## 4. the protocol

### 4.1 program

```json
{
  "schema": "sovware.dev/program/v1",
  "id": "mechanical/bracket",
  "version": 4,
  "domain": "mechanical",
  "kind": "cadquery.part",
  "source": { "path": "bracket.py", "lang": "python", "sha256": "sha256:…" },
  "params": {
    "length_mm": { "value": 120, "unit": "mm", "min": 40, "max": 400 },
    "material":  { "value": "6061-T6" }
  },
  "claims": [
    { "id": "C1", "text": "M6 clearance hole", "source": { "kind": "standard", "ref": "ISO 273" } }
  ],
  "budget": { "wall_s": 60, "usd": 0.05, "mass_g": 250 },
  "parents": ["mechanical/bracket@3"],
  "derived": [{ "path": "bracket.step", "sha256": "sha256:…", "label": "derived" }],
  "provenance": { "actor": "agent:nightfall", "model": "…", "at": "2026-10-20T12:00:00Z" }
}
```

### 4.2 verifier

```json
{
  "schema": "sovware.dev/verifier/v1",
  "id": "sov/units",
  "version": "0.1.0",
  "kind": "exact",
  "gates": true,
  "applies_to": { "domains": ["*"], "kinds": ["*"] },
  "invoke": { "cmd": ["python3", "main.py"], "input": "stdin", "timeout_s": 30 },
  "cost": { "wall_s": 1, "usd": 0 },
  "license": "Apache-2.0",
  "checks": ["units are declared", "bounds contain value", "tolerance is non-negative"]
}
```

### 4.3 result

```json
{
  "schema": "sovware.dev/verifier-result/v1",
  "verifier": "sov/units",
  "version": "0.1.0",
  "kind": "exact",
  "verdict": "fail",
  "score": 0.0,
  "findings": [
    { "level": "error", "code": "SOV-UNIT-001", "message": "length_mm has no unit", "path": "params.length_mm" }
  ],
  "measures": {},
  "cost": { "wall_ms": 12 }
}
```

Rules:

- stdout carries exactly one JSON object. stderr is for humans and is logged, not parsed.
- exit code is advisory. The verdict is authoritative. A non-zero exit with a valid `fail`
  result is a fail, not an error.
- a crash, timeout, malformed result, or unknown `verifier` id is `error`.
- a verifier MUST NOT write outside its declared `invoke.workdir`.
- a verifier MUST be deterministic given the same program bytes and environment. A verifier
  that is not MUST declare `kind: statistical`.

---

## 5. the gate

A domain declares its gate in `schema/domain.schema.json`:

```json
{
  "id": "mechanical",
  "gate": {
    "require": ["sov/units", "sov/claims", "mech/yield-margin"],
    "min_score": { "mech/yield-margin": 0.8 },
    "on_error": "fail"
  }
}
```

Evaluation order:

1. every verifier in `require` MUST resolve by `id`, and MUST match `applies_to`;
2. every gating verifier MUST return `pass` (or `numeric` with `score ≥ min_score`);
3. any `error`, `skip`, or `missing` is a refusal unless `on_error` is explicitly `report`
   and the run is marked `advisory` (never used for fabrication);
4. the gate result is recorded with the per-verifier results and their hashes.

A gate with fewer than two independent exact verifiers is a `draft` gate and MUST NOT be
used for a published claim.

---

## 6. the ledger

One JSON object per line. Required fields: `id`, `program`, `program_hash`, `domain`, `gate`,
`results`, `cost`, `actor`, `created`, `prev`, `hash`.

- `hash = "sha256:" + sha256(canonical_json(run without hash))`, keys sorted, no whitespace.
- `prev` equals the previous line's `hash` in the same ledger, or `null`.
- `sov ledger verify` recomputes every hash and every link.
- The ledger is the only accepted evidence in a report. Screenshots are not evidence.

---

## 7. the improvement protocol

A mutation `M` of parent `P` is an improvement iff:

1. `gate(M) = pass`;
2. `score(M) ≥ score(P)` on the held-out set `H`, where `H` is fixed before `M` exists and
   `M`'s author cannot read it;
3. `cost(M) ≤ cost(P)` on the declared axis.

`score` is the domain's primary measure, declared in the domain file. It is not an average
of verifier scores unless the domain says so.

The protocol:

1. freeze `H` and publish its hash (`schema/eval.schema.json`, `holdout_hash`);
2. record `P` on `H`;
3. generate `M` with no access to `H`;
4. record `M` on `H`;
5. compare. If pass, accept and publish both runs. If not, keep the record and say
   `change` or `regression`.

`sov eval` implements this: it hashes the holdout cases, refuses when they no longer match
the published `holdout_hash`, runs every case through the same gate as `sov verify`, and
writes a report that includes the failures. A sealed set is the only one whose result counts
as evidence.

A domain SHOULD keep a `train` set and MUST keep `H` disjoint from it. Overfitting on `train`
is expected and uninteresting. Publishing `train` as `H` is fraud.

---

## 8. domain admission

A domain is admitted when it ships:

- `domain.json` conforming to the schema, with `artifacts`, `gate`, `budget`, `non_goals`;
- at least two exact verifiers and one numeric or simulation verifier;
- a corpus of at least 30 programs, of which at least 10 are in the held-out set;
- five known-bad programs that the gate rejects, with the failing verifier named;
- a reproduction command that runs offline.

A domain without known-bad examples has not been tested. It has been demonstrated.

---

## 9. conformance

| Level | Requires |
| --- | --- |
| **L0 — schema** | programs, verifiers and runs validate; the kernel parses them strictly |
| **L1 — loop** | `sov verify` runs a gate, appends a ledger, and the chain verifies |
| **L2 — domain** | §8 satisfied for one domain, with held-out results published |
| **L3 — improvement** | one mutation accepted under §7, with both runs linked |

The reference implementation in `crates/` targets L1 and demonstrates L2–L3 on the bracket
example.

---

## 10. security model

- Verifiers are untrusted. They run with a timeout, in a working directory they own, and
  with no network unless the verifier declares `network: true` and the domain gate allows
  it.
- A verifier declaring `network: true` MUST NOT gate a fabrication path.
- Programs are data. A program MUST NOT be executed to render it. Rendering happens inside
  a verifier that declares `executes_source: true`.
- The kernel has no credential store. Model keys live in `invincible`; the kernel sees
  neither.
- Ledger hashes are integrity, not identity. Signing is the host's job.

---

## 11. change control

- Schema changes are additive within a major version. A breaking change increments the
  `schema` string (`/v1` → `/v2`) and ships a migration in `migrations/`.
- A requirement is removed only by an ADR in `docs/adr/` that names what it protected.
- The manifesto outranks this document. This document outranks code. Code outranks
  opinion.

---

## appendix a — what happened to ASC-SYS

| ASC-SYS | Disposition |
| --- | --- |
| §1.2 Deterministic Validation | kept, promoted to R2/R8 and §5 |
| §1.2 First-Principles Reduction | kept as a research method, not a requirement: it has no test |
| §1.2 Resource Bound Optimization | kept as R10, with a cost object and a budget gate |
| Phase I Discovery | demoted to a producer of `claims` (R3); not on the critical path |
| Phase II Mutation | kept, but after the gate exists; §7 defines acceptance |
| Phase III Simulation | kept as verifiers of kind `numeric`, out of process (R6) |
| Phase IV Compliance | split: machine-checkable rules become verifiers; sign-off becomes R9 |
| Phase V Telemetry | replaced by the ledger (R7); dashboards are downstream, not evidence |
| Vendor matrix | deleted. Replaced by `docs/FORKS.md`, which is a build/buy/wrap decision record, not an architecture |
