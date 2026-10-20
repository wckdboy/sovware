#!/usr/bin/env python3
"""sov/claims — exact. Nothing is true because a model said it.

A claim without a resolvable source is a bug, not a style problem (SPEC R3).
"""

import json
import re
import sys

# Each source kind has a shape we can resolve without the network. The kernel runs
# offline; a reference that only resolves online is not evidence here.
SHAPES = {
    "standard": re.compile(r"^[A-Za-z][A-Za-z0-9 .:/-]*\d[A-Za-z0-9 .:/-]*$"),
    "paper": re.compile(r"^(arXiv:\d{4}\.\d{4,5}|doi:10\.\d{4,9}/\S+|https://\S+)$"),
    "test": re.compile(r"^(test://\S+|https://\S+|ledger:run_[a-f0-9]{12})$"),
    "derivation": re.compile(r"^(derivation:\S+|\S+\s*=>\s*\S+)$"),
    "measurement": re.compile(r"^(measurement://\S+|https://\S+|ledger:run_[a-f0-9]{12})$"),
}


def finding(level, code, message, path=None, hint=None):
    f = {"level": level, "code": code, "message": message}
    if path:
        f["path"] = path
    if hint:
        f["hint"] = hint
    return f


def check(program):
    findings = []
    claims = program.get("claims") or []
    params = program.get("params") or {}
    measures = {"claims": len(claims), "cited": 0}

    if not claims:
        findings.append(finding(
            "error", "SOV-CLAIM-001",
            "program declares no claims",
            "claims",
            "state at least one fact the artifact depends on, with a source",
        ))

    seen = set()
    for i, claim in enumerate(claims):
        path = f"claims[{i}]"
        cid = claim.get("id")
        if not cid:
            findings.append(finding("error", "SOV-CLAIM-002", "claim has no id", path))
            continue
        if cid in seen:
            findings.append(finding("error", "SOV-CLAIM-002", f"duplicate claim id {cid}", path))
        seen.add(cid)

        if not (claim.get("text") or "").strip():
            findings.append(finding("error", "SOV-CLAIM-002", f"claim {cid} has no text", path))

        source = claim.get("source") or {}
        kind = source.get("kind")
        ref = (source.get("ref") or "").strip()
        pattern = SHAPES.get(kind)
        if pattern is None:
            findings.append(finding(
                "error", "SOV-CLAIM-003",
                f"claim {cid} has unknown source kind {kind!r}", path,
                f"one of: {', '.join(SHAPES)}",
            ))
        elif not pattern.match(ref):
            findings.append(finding(
                "error", "SOV-CLAIM-003",
                f"claim {cid} has unresolvable {kind} reference {ref!r}", path,
                f"must match {pattern.pattern}",
            ))
        else:
            measures["cited"] += 1

    for name, param in params.items():
        if not isinstance(param, dict):
            continue
        ref = param.get("claim")
        if ref is None:
            continue
        if ref not in seen:
            findings.append(finding(
                "error", "SOV-CLAIM-004",
                f"{name} cites claim {ref!r}, which does not exist",
                f"params.{name}",
            ))

    return findings, measures


def main():
    program = json.load(sys.stdin)
    findings, measures = check(program)
    errors = [f for f in findings if f["level"] == "error"]
    result = {
        "schema": "sovware.dev/verifier-result/v1",
        "verifier": "sov/claims",
        "version": "0.1.0",
        "kind": "exact",
        "verdict": "fail" if errors else "pass",
        "score": 0.0 if errors else 1.0,
        "findings": findings,
        "measures": measures,
    }
    json.dump(result, sys.stdout)
    sys.stdout.write("\n")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
