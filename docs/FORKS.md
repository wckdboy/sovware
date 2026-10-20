# forks, dependencies and the license boundary

This is the build/buy/wrap decision record. It replaces the vendor matrix in
`ASC-SYS-2026-V4.2`, which listed products as if they were architecture.

The rule from the manifesto (§5): **we fork only when upstream is dead, the license forbids
the use, or the change is not wanted. Fixes go back first.**

Every entry below is one of five verdicts:

| Verdict | Meaning |
| --- | --- |
| **depend** | a library we link; must be permissive enough to ship |
| **wrap** | an executable we call; any license, any language, out of process |
| **upstream** | we need a change and will send it to the project first |
| **fork** | we maintain a divergent copy, with `PATCHES.md` and a rebase cadence |
| **build** | nobody has it, or what exists cannot be trusted with a score |

## the boundary is the license strategy

A verifier is a subprocess (`SPEC R6`). That is not only fault isolation. It is how a
GPL-3.0 solver, an AGPL slicer, an LGPL kernel and a proprietary vendor SDK can share one
gate without touching the kernel's license, address space, or crash domain.

- **Linked** code must be Apache-2.0 compatible. The kernel and its crates are
  Apache-2.0.
- **Called** code can be anything. We pass a program in, we get a verdict out.
- **Served** code (a search instance, a dashboard) is never a dependency of a gate. If a
  gate needs the network, it is not a gate (`SPEC §10`).

Two licenses worth naming because they are commonly mislabelled: Arize Phoenix is
Elastic-2.0 (source-available, not OSI open source), and Grafana, OctoPrint, OrcaSlicer,
Cura, PrusaSlicer and SearXNG are AGPL-3.0. All are fine behind a process boundary. None
may be linked into a binary we ship under a permissive license.

## the register

### kernel

| Component | Verdict | License | Note |
| --- | --- | --- | --- |
| Rust, serde, sha2, clap, chrono | depend | MIT / Apache-2.0 | no async runtime in the kernel |
| JSON Schema 2020-12 | spec | — | normative; Rust types mirror it |
| `omnie-harness` | depend | MIT | job, step, checkpoint, resume. The kernel does not reimplement scheduling. |
| `invincible` | wrap | MIT | model routing and receipts; keys never reach the kernel |

### verification and execution

| Component | Verdict | License | Note |
| --- | --- | --- | --- |
| gVisor | wrap | Apache-2.0 | untrusted verifier isolation on Linux |
| Firecracker | wrap | Apache-2.0 | microVM option; heavier, stronger |
| E2B | wrap | Apache-2.0 | self-host only; a hosted interpreter is not a gate |
| landlock / seccomp | build | kernel | filesystem and syscall limits for verifiers |
| OpenTelemetry | depend | Apache-2.0 | traces out of the ledger, never instead of it |

### mechanical, civil, aerospace

| Component | Verdict | License | Note |
| --- | --- | --- | --- |
| CadQuery | wrap | Apache-2.0 | Python API, headless; `bracket.py` is written against it |
| OCCT | wrap | LGPL-2.1 + exception | geometry kernel under CadQuery; dynamic link only |
| build123d | watch | Apache-2.0 | arguably a better scripted API than CadQuery; decide on evidence |
| Gmsh | wrap | GPL-2.0+ | meshing for FEA/CFD |
| CalculiX | wrap | GPL-2.0 | linear/nonlinear FEA; small, fast, scriptable |
| Code_Aster | wrap | GPL-2.0 | when CalculiX is not enough |
| FEniCSx | wrap | LGPL-3.0 | research-grade PDE; Python-native |
| SU2 | wrap | LGPL-2.1 | compressible CFD, adjoints for shape optimization |
| OpenFOAM | wrap | GPL-3.0 | the broad CFD fallback |
| openVSP | wrap | NOSA-1.3 | aircraft geometry; OSI-approved, unusual terms |
| Orekit | depend | Apache-2.0 | orbital mechanics, Java; call it as a service |
| OpenRocket | wrap | GPL-3.0 | hobby-scale trajectory cross-check |

### electrical

| Component | Verdict | License | Note |
| --- | --- | --- | --- |
| KiCad | wrap | GPL-3.0 | `kicad-cli pcb drc` and ERC; the default EDA gate |
| skidl | wrap | MIT | netlist as Python; good for generated circuits |
| ngspice | wrap | BSD-3-Clause | simulation; clean license, easy gate |
| OpenLane | watch | Apache-2.0 | ASIC flow; too heavy until a domain needs it |

### fabrication

| Component | Verdict | License | Note |
| --- | --- | --- | --- |
| OrcaSlicer | wrap | AGPL-3.0 | headless slicing; version must be pinned in the verifier |
| PrusaSlicer / Cura | wrap | AGPL-3.0 | same role, different profiles |
| Klipper / OctoPrint | wrap | GPL-3.0 / AGPL-3.0 | **never called by the kernel.** Only a human-signed host (`R9`) may call these |
| LinuxCNC / GRBL | wrap | GPL-2.0 / GPL-3.0 | same rule |

### discovery and knowledge

| Component | Verdict | License | Note |
| --- | --- | --- | --- |
| arXiv API | depend | terms | metadata and LaTeX where the license allows; record the license per item |
| OpenAlex | depend | CC0 | the citation graph; no scraping |
| Crossref | depend | open metadata | DOIs |
| Unpaywall | depend | CC0 metadata | open-access locations only |
| SearXNG | wrap | AGPL-3.0 | self-hosted search; a service, not a linked library |
| LlamaIndex | watch | MIT | adopt only if a real graph query needs it |
| Microsoft GraphRAG | watch | MIT | heavy; the ontology in `domains/` is deliberately small |
| oxigraph | watch | MIT/Apache-2.0 | embeddable RDF store, if the graph outgrows SQLite |

### prompts and models

| Component | Verdict | License | Note |
| --- | --- | --- | --- |
| DSPy | depend | MIT | prompts as programs; the manifesto's §1 and §4 in library form |
| GEPA | depend | MIT | reflective mutation, cheaper than gradient-style search |
| promptfoo | wrap | MIT | red-team and regression suites for prompt topologies |
| Inspect | watch | MIT | model evals; does not replace domain verifiers |
| MLflow | watch | Apache-2.0 | experiment tracking, if the ledger is not enough (it is) |

## fork candidates

Ordered by how much they block the first three domains.

### 1. deterministic STEP export — upstream, then fork if refused

STEP writers embed a timestamp, an author and the originating filename in `FILE_NAME`.
Two exports of identical geometry therefore hash differently, which breaks `R1` and the
ledger. CadQuery/OCCT expose the writer but not a "canonical" mode.

**Plan:** patch the export path used by our CAD verifier to zero the header, then offer the
patch upstream to CadQuery and OCCT. If refused, fork the thin writer, not the kernel.

**This is the first patch we owe upstream**, and it is small.

### 2. headless DRC with a stable JSON contract — wrap KiCad, contribute the contract

`kicad-cli pcb drc` exists and can emit a report, but the JSON shape moves between major
versions. Our verifier needs a stable, versioned result.

**Plan:** a thin adapter in `verifiers/kicad-drc/` that pins the KiCad version, parses the
report, and emits `verifier-result/v1`. Contribute the schema mapping upstream if KiCad
wants a stable machine contract. Do not fork KiCad for this.

### 3. deterministic slicing — watch OrcaSlicer

Slicing output depends on profiles, printer definitions and version. A gate needs all three
pinned, plus a G-code analyzer that checks extrusion, layer time and travel without opening
a GUI.

**Plan:** wrap the CLI, pin the appimage digest, write the analyzer. Fork only if the CLI
cannot be made deterministic. AGPL stays behind the process boundary either way.

### 4. CadQuery's mass and bounding-box properties — upstream

`Shape.Volume()`, `Shape.val().BoundingBox()` and material density are enough for a first
mass budget, but there is no material-aware mass property and no tolerance on the volume.
Small upstream contribution. Do not fork.

### 5. `omnie-dev-editor-engine` — standing fork, already running

The precedent for how we do this: a hard fork of Runestone, `upstream` branch mirrors the
original, `main` carries `PATCHES.md`, and fixes worth sharing go upstream first. Same
structure applies to anything in this list that reaches **fork**.

## what we will not fork

- **Ansys, Altium, SimScale, Synera, Navier, Leo, Tavily, Exa, LangSmith.** Closed, and the
  point of the estate is that the loop runs without them. They may be *called* by a user
  who owns a license; they may never be a gate the project depends on.
- **A benchmark.** We do not fork an eval to make our numbers look better. That is the one
  fork that would make every other number meaningless.
- **The kernel, into a domain.** If a domain needs a behaviour the kernel does not have,
  the kernel gets a requirement, not a copy.

## how a fork is recorded

Every fork gets, in its own repository:

1. `PATCHES.md` — one entry per divergence: what, why, upstream issue link, and the cost to
   rebase.
2. an `upstream` branch that tracks the source exactly, so `git diff upstream..main` is the
   whole debt.
3. a rebase cadence and a named owner.
4. a line in this file with the license and the boundary.

A fork without those four is a vendored copy with extra steps, and it will rot.
