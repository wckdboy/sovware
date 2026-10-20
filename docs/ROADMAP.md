# roadmap

The roadmap is ordered by what would make the loop dishonest if it were missing, not by
what demos well.

## now — the kernel (L1, done)

- [x] six schemas, strict parsing, unknown fields rejected
- [x] `sov verify`, `sov compare`, `sov ledger verify`
- [x] hash-chained ledger, verifier definition hashes
- [x] timeout, error-is-refusal, human-signature gate
- [x] one domain with known-bad programs and a real improvement
- [ ] `sov eval` — run an eval set, enforce holdout/hash, emit a report that includes failures
- [ ] verifier sandbox: landlock + seccomp on Linux, `sandbox-exec` on macOS
- [ ] parallel verifiers with per-verifier budget accounting

## next — two more domains, chosen for their gates

### circuits

Gate: KiCad ERC/DRC (`kicad-cli`), netlist connectivity, ngspice DC/AC operating point,
and a current-density check on copper.

Blocker: a stable machine contract for KiCad's reports (`docs/FORKS.md` §2). Until then the
verifier pins a KiCad version and parses defensively.

### software as a program

Gate: build, typecheck, test suite on a held-out corpus, argument-level property tests.

Why it matters: this is the domain that makes the manifesto recursive. The agent mutates
its own prompts and routing, and the same ledger decides whether it improved. The verifiers
are compilers and tests, which already exist and already fail loudly.

Blocker: a story for the cost axis. Wall time is not enough; tokens and dollars are. Use
`invincible` receipts as the cost source.

## later — the expensive gates

- **fluids:** SU2 or OpenFOAM; the first verifier whose wall time is minutes, not
  milliseconds. This is where `wckd-gpu` becomes the compute substrate and the ledger's
  cost field becomes load-bearing.
- **structures beyond a lug:** CalculiX, with a mesh convergence check as its own verifier.
  A solver result without convergence evidence is a statistical claim wearing a numeric
  label.
- **orbits:** Orekit, with a propagator cross-check.

## never on the roadmap

- autonomous fabrication (`R9`)
- a hosted oracle in the critical path (manifesto §12)
- a leaderboard (manifesto §11)
