# contributing

The project is small on purpose. The kernel has four objects and aims to keep them.

## what is welcome

- **Verifiers.** One executable, one claim about the world, one result. A good verifier is
  boring, deterministic, and fails on a specific defect you can name.
- **Known-bad programs.** More valuable than passing ones. If you can break a gate, that is
  a contribution to the gate.
- **A domain.** Follow `docs/SPEC.md` §8. Do not open one without the corpus.
- **Upstream patches.** If you fix something in CadQuery, KiCad, SU2 or OrcaSlicer, link
  the upstream PR here and we will drop the local workaround.

## what is not

- A new abstraction in the kernel. Four objects is the budget. If you need a fifth, open an
  ADR in `docs/adr/` and name what it protects.
- A vendor integration presented as a capability. See `docs/FORKS.md` for the difference
  between *called* and *depended on*.
- A benchmark result without the ledger run that produced it.
- Generated code, prose, or geometry without a claim and a source.

## process

1. Open an issue with the failure, not the feature. What did the system accept that it
   should have refused?
2. Add the failing case first. It must fail for the reason you say.
3. Change the smallest thing that makes it fail correctly.
4. `make check`.
5. One commit, one idea. Sign off if your employer requires it (`git commit -s`).

There is no CLA. Contributions are Apache-2.0, inbound equals outbound.

## adding a domain

A domain PR includes:

- `domains/<id>.json`, with non-goals that are specific enough to be embarrassing;
- the verifier set, each with `kind`, `gates`, `license` and `checks`;
- 30 programs, 10 held out, 5 known-bad;
- a `README.md` in the domain folder that states the assumptions the verifiers make, in
  plain language, so a reviewer can attack them.

If the assumptions cannot be written in a paragraph, the domain is not ready.

## security

Verifiers are untrusted code. Do not add one that needs credentials, and do not add one
that declares `network: true` to a fabrication path (`SPEC §10`). Report a kernel escape
privately to security@sovware.io before opening an issue.
