#!/usr/bin/env python3
"""sov/units — exact. Every number carries its unit, and its bounds contain it.

Reads one program on stdin, writes one verifier result on stdout.
"""

import json
import sys

# A small, explicit registry. Unknown units fail: a unit we cannot read is a unit we
# cannot check, and inventing a conversion is worse than refusing.
UNITS = {
    "1": "dimensionless", "%": "ratio",
    "nm": "length", "um": "length", "mm": "length", "cm": "length", "m": "length",
    "in": "length", "ft": "length",
    "mg": "mass", "g": "mass", "kg": "mass", "t": "mass",
    "ms": "time", "s": "time", "min": "time", "h": "time",
    "N": "force", "kN": "force", "MN": "force",
    "Pa": "stress", "kPa": "stress", "MPa": "stress", "GPa": "stress",
    "J": "energy", "kJ": "energy", "W": "power", "kW": "power",
    "V": "voltage", "A": "current", "mAh": "charge", "ohm": "resistance",
    "Hz": "frequency", "kHz": "frequency", "MHz": "frequency",
    "mm2": "area", "mm3": "volume", "mm4": "second moment",
    "deg": "angle", "rad": "angle",
    "C": "temperature", "K": "temperature",
}


def finding(level, code, message, path=None, hint=None):
    f = {"level": level, "code": code, "message": message}
    if path:
        f["path"] = path
    if hint:
        f["hint"] = hint
    return f


def check(params):
    findings = []
    measures = {"params": len(params), "typed": 0}

    if not params:
        findings.append(finding("error", "SOV-UNIT-000", "program declares no parameters"))
        return findings, measures

    for name, param in params.items():
        path = f"params.{name}"

        if not isinstance(param, dict):
            if isinstance(param, bool):
                continue
            if isinstance(param, (int, float)):
                findings.append(finding(
                    "error", "SOV-UNIT-001",
                    f"{name} is a bare number",
                    path,
                    "use {\"value\": <n>, \"unit\": \"mm\"}",
                ))
            continue

        measures["typed"] += 1
        value = param.get("value")
        unit = param.get("unit")
        minimum = param.get("min")
        maximum = param.get("max")
        tolerance = param.get("tolerance")

        numeric = isinstance(value, (int, float)) and not isinstance(value, bool)
        if numeric and not unit:
            findings.append(finding(
                "error", "SOV-UNIT-001",
                f"{name} is numeric with no unit",
                path,
                "add \"unit\": \"mm\" or \"1\" for dimensionless",
            ))
        if unit is not None and unit not in UNITS:
            findings.append(finding(
                "error", "SOV-UNIT-002",
                f"{name} has unknown unit {unit!r}",
                path,
                f"known: {', '.join(sorted(UNITS))}",
            ))

        if minimum is not None and maximum is not None and minimum > maximum:
            findings.append(finding(
                "error", "SOV-UNIT-003",
                f"{name} has min {minimum} > max {maximum}",
                path,
            ))
        if numeric and minimum is not None and value < minimum:
            findings.append(finding(
                "error", "SOV-UNIT-004",
                f"{name}={value} is below min {minimum}",
                path,
            ))
        if numeric and maximum is not None and value > maximum:
            findings.append(finding(
                "error", "SOV-UNIT-004",
                f"{name}={value} is above max {maximum}",
                path,
            ))
        if tolerance is not None and (not isinstance(tolerance, (int, float)) or tolerance < 0):
            findings.append(finding(
                "error", "SOV-UNIT-005",
                f"{name} has invalid tolerance {tolerance!r}",
                path,
            ))

    return findings, measures


def main():
    program = json.load(sys.stdin)
    findings, measures = check(program.get("params") or {})
    errors = [f for f in findings if f["level"] == "error"]
    result = {
        "schema": "sovware.dev/verifier-result/v1",
        "verifier": "sov/units",
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
