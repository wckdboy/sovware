# the sovware manifesto

*wckd lab · sovware.io · v1.0 · 2026-10-20*

We build tools for two kinds of builders: people, and the agents people work with.
One interface, both citizens. The human can read everything the agent can do, and the
agent can do everything the human can do, except the things that move matter.

The tools we are replacing are confident. They produce a part that looks right, a board
that looks routable, a plan that reads well. The failure is never at the surface. It is
under the surface, where nothing checked.

So we start from the check.

---

## 1. the artifact is a program

A part is not a shape. A board is not a picture. A wing is not a mesh. Each is a program:
named parameters, typed units, declared tolerances, a lineage, and source you can read.

If it cannot be re-derived from its source, we do not call it an artifact. Meshes are
caches. Renders are evidence of nothing.

## 2. the verifier is the oracle

An agent has no ground truth until we give it one. A verifier is an executable that takes
a program and returns a verdict, a score, a reason, and a cost.

A verifier is exact, numeric, or statistical, and it says which. Exact verifiers gate.
Numeric verifiers gate against a declared tolerance. Statistical verifiers inform and never
gate alone. A verifier that cannot state its own uncertainty has no vote.

## 3. nothing is true because a model said it

Models generate candidates. They do not generate facts. Every claim in an artifact points
at a source: a standard, a paper, a measurement, a test run. No source, no claim. A
citation that cannot be resolved is a bug, not a style problem.

## 4. the agent edits the program, never the result

Geometry is a cache. Pixels are a cache. Prompt text is source. The agent changes source
and the system rebuilds. This is the only way a human can review what changed, the only way
a change can be rolled back, and the only way two agents can work on one artifact.

## 5. domains are added by verifiers, not vendors

We do not enter a domain because a company sells into it. We enter a domain when we can
score it. Civil engineering is not a subscription. It is a set of checkers, tolerances and
codes that we can implement, cite, and test.

Where a good open tool exists, we call it. Where it nearly exists, we improve it upstream.
We fork only when upstream is dead, the license forbids the use, or the change is not
wanted. Fixes go back first. A fork is a debt with an address.

## 6. the boundary is a process, not a language

Every verifier runs out of process, behind a protocol. That is how a GPL solver, a Python
kernel, a vendor SDK and a shell script can share one gate without sharing a license, a
heap, or a crash. The kernel stays small and permissive. The world stays plural.

## 7. the ledger is the memory

Every run is recorded: the program, its hash, every verifier and version, every score,
every cost, the actor, and the hash of the run before it. Append-only, replayable, and
diffable. The ledger is not telemetry. Telemetry is for dashboards. The ledger is the
evidence, and it is what the next mutation learns from.

## 8. cost is a constraint, not a dashboard

Every artifact carries a budget: wall time, dollars, material, mass. A mutation that is
better and more expensive is a trade, not an improvement, and it must be named as one.
Token counts are not the bottleneck. Solver iterations, print hours and board spins are.

## 9. the loop is measured

A mutation is an **improvement** if and only if all three hold:

1. it passes the domain gate;
2. it scores at least as well as its parent on a held-out set it was not tuned on;
3. it costs no more than its parent on the axis the domain declared.

Everything else is a **change**. We keep the improvements. We keep the record of the
changes. We never call a change an improvement, including when it is ours.

## 10. the machine has a human gate

Anything that moves matter — G-code, a fabrication order, a PCB run — stops at a person.
Not because the verifiers are weak, but because a verifier is a program and matter does not
roll back. The gate is one signature wide. We make it fast, not optional.

## 11. the record is public

Scores, failures, regressions and the times the agent was confidently wrong. A benchmark we
tuned on is not evidence. A demo we cannot rerun is an anecdote.

## 12. sovereign by construction

The loop runs on hardware you own, with the network off, without a vendor's permission.
Frontier models are an accelerant, never a dependency. If a critical path needs someone
else's server to say yes, it is not ours.

## 13. the test

If we cannot take one part, in one domain, on one machine, with the network unplugged, run
the loop, and show the score go up — we have built a slide.

That is the test. Everything in this repository serves it.

---

## what we refuse

- A vendor list presented as an architecture.
- Autonomous fabrication. The last step is a person.
- A claim without a verifier, and a verifier without a declared kind.
- A closed oracle in the critical path.
- A benchmark tuned on, quoted as a result.
- The word "open" for something whose source you cannot read.
- "Deterministic" for anything a model generates.
- Shipping the demo and hiding the failure rate.

## the first slice

One domain: mechanical. One artifact: a bracket. One exact verifier: units and bounds.
One numeric verifier: margin against yield. One ledger. One mutation that earns its place.

Then circuits. Then the rest.

---

*The manifesto is normative. Its requirements are numbered `R1`–`R12` in
[`docs/SPEC.md`](docs/SPEC.md), and a change that violates one is not merged, however good
the demo looks.*
