#!/usr/bin/env python3
"""sov/source — exact. If the artifact cannot be re-derived from the bytes named,
the program is not an artifact (SPEC R1).

The kernel sets SOV_PROGRAM_DIR to the directory holding the program file.
"""

import hashlib
import json
import os
import sys
from pathlib import Path


def finding(level, code, message, path=None, hint=None):
    f = {"level": level, "code": code, "message": message}
    if path:
        f["path"] = path
    if hint:
        f["hint"] = hint
    return f


def inside(base, candidate):
    try:
        candidate.relative_to(base)
        return True
    except ValueError:
        return False


def digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return "sha256:" + h.hexdigest()


def main():
    program = json.load(sys.stdin)
    findings = []
    measures = {"checked": 0}
    base = Path(os.environ.get("SOV_PROGRAM_DIR", ".")).resolve()

    source = program.get("source") or {}
    if source.get("inline"):
        findings.append(finding(
            "warning", "SOV-SOURCE-003",
            "inline source is recorded but not verified",
            "source.inline",
        ))
    else:
        rel = source.get("path")
        if not rel:
            findings.append(finding("error", "SOV-SOURCE-001", "source has no path", "source.path"))
        else:
            target = (base / rel).resolve()
            if not inside(base, target):
                findings.append(finding(
                    "error", "SOV-SOURCE-001",
                    f"source path {rel!r} escapes the program directory",
                    "source.path",
                ))
            elif not target.is_file():
                findings.append(finding(
                    "error", "SOV-SOURCE-001",
                    f"source {rel!r} does not exist",
                    "source.path",
                ))
            else:
                actual = digest(target)
                measured = source.get("sha256")
                measures["checked"] += 1
                measures["actual"] = actual
                if actual != measured:
                    findings.append(finding(
                        "error", "SOV-SOURCE-002",
                        f"source hash mismatch: declared {measured}, on disk {actual}",
                        "source.sha256",
                        "the program describes different bytes than the ones present",
                    ))

    for i, item in enumerate(program.get("derived") or []):
        path = f"derived[{i}]"
        if item.get("label") != "derived":
            findings.append(finding(
                "error", "SOV-SOURCE-004",
                "derived artifact is not labelled derived", path,
            ))
        rel = item.get("path")
        if rel and (base / rel).is_file():
            if digest((base / rel).resolve()) != item.get("sha256"):
                findings.append(finding(
                    "error", "SOV-SOURCE-004",
                    f"derived artifact {rel!r} hash mismatch", path,
                ))
        else:
            findings.append(finding(
                "error", "SOV-SOURCE-004",
                f"derived artifact {rel!r} is missing", path,
            ))

    errors = [f for f in findings if f["level"] == "error"]
    result = {
        "schema": "sovware.dev/verifier-result/v1",
        "verifier": "sov/source",
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
