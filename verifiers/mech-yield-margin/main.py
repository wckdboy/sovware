#!/usr/bin/env python3
"""mech/yield-margin — numeric. A flat lug under axial load, checked three ways.

The first domain verifier in the estate, on purpose: hand calculation, no solver, no
dependency, reproducible offline. When the loop is honest here, it can carry a solver.

Assumptions, stated so they can be attacked:
  - the load is axial and shared equally by `hole_count` holes in one cross-section;
  - the plate is flat, constant thickness, and the net section is W - n*d;
  - allowables are handbook minima for the named alloy, not a certified allowables
    database. This verifier is a gate, not a sign-off.
"""

import json
import sys

Y = "yield"

# yield strength MPa, density g/mm3, bearing factor, shear factor
MATERIALS = {
    "6061-T6": {"sy": 276.0, "rho": 2.70e-3, "bearing": 1.5, "shear": 0.60},
    "7075-T6": {"sy": 503.0, "rho": 2.81e-3, "bearing": 1.5, "shear": 0.60},
    "304":     {"sy": 215.0, "rho": 8.00e-3, "bearing": 1.5, "shear": 0.60},
    "A36":     {"sy": 250.0, "rho": 7.85e-3, "bearing": 1.5, "shear": 0.60},
    "PLA":     {"sy": 50.0,  "rho": 1.24e-3, "bearing": 1.5, "shear": 0.50},
    "PETG":    {"sy": 47.0,  "rho": 1.27e-3, "bearing": 1.5, "shear": 0.50},
}

REQUIRED = [
    "length_mm", "width_mm", "thickness_mm", "hole_d_mm",
    "hole_count", "edge_mm", "load_n", "material", "safety_factor",
]


def finding(level, code, message, path=None, hint=None):
    f = {"level": level, "code": code, "message": message}
    if path:
        f["path"] = path
    if hint:
        f["hint"] = hint
    return f


def value(param):
    return param.get("value") if isinstance(param, dict) else param


def clamp(x, lo, hi):
    return max(lo, min(hi, x))


def main():
    program = json.load(sys.stdin)
    params = program.get("params") or {}
    findings = []
    measures = {}

    missing = [k for k in REQUIRED if k not in params]
    if missing:
        for name in missing:
            findings.append(finding(
                "error", "MECH-INPUT-001", f"missing parameter {name}", f"params.{name}",
            ))
        return emit(findings, measures, 0.0)

    try:
        length = float(value(params["length_mm"]))
        width = float(value(params["width_mm"]))
        thickness = float(value(params["thickness_mm"]))
        hole_d = float(value(params["hole_d_mm"]))
        holes = int(value(params["hole_count"]))
        edge = float(value(params["edge_mm"]))
        load = float(value(params["load_n"]))
        material_name = str(value(params["material"]))
        sf = float(value(params["safety_factor"]))
    except (TypeError, ValueError) as exc:
        findings.append(finding("error", "MECH-INPUT-002", f"non-numeric input: {exc}"))
        return emit(findings, measures, 0.0)

    material = MATERIALS.get(material_name)
    if material is None:
        findings.append(finding(
            "error", "MECH-INPUT-003",
            f"unknown material {material_name!r}", "params.material",
            f"known: {', '.join(sorted(MATERIALS))}",
        ))
        return emit(findings, measures, 0.0)
    if sf < 1.0:
        findings.append(finding("error", "MECH-INPUT-004", f"safety_factor {sf} < 1", "params.safety_factor"))

    # Geometry: the net section must exist and the holes must fit with an edge margin.
    if holes * hole_d + 2 * edge > width:
        findings.append(finding(
            "error", "MECH-GEO-001",
            f"{holes} x {hole_d} mm holes plus {edge} mm edges do not fit in {width} mm",
            "params.width_mm",
        ))
    if edge < 1.5 * hole_d:
        findings.append(finding(
            "error", "MECH-GEO-002",
            f"edge distance {edge} mm < 1.5 d ({1.5 * hole_d:.2f} mm)",
            "params.edge_mm",
            "tear-out governs before bearing",
        ))
    if thickness <= 0 or width <= 0 or hole_d <= 0:
        findings.append(finding("error", "MECH-GEO-003", "non-positive dimension"))

    net_width = width - holes * hole_d
    if net_width <= 0:
        findings.append(finding("error", "MECH-GEO-001", "net section is zero or negative", "params.width_mm"))
        return emit(findings, measures, 0.0)

    sy = material["sy"]
    bearing_allow = material["bearing"] * sy
    shear_allow = material["shear"] * sy

    demands = {
        "net-section": (load / (thickness * net_width), sy, "MECH-YIELD-001"),
        "bearing": (load / (holes * hole_d * thickness), bearing_allow, "MECH-YIELD-002"),
        "tear-out": (load / (2 * holes * edge * thickness), shear_allow, "MECH-YIELD-003"),
    }

    margins = {}
    governing = ("", float("inf"))
    for name, (demand, allow, code) in demands.items():
        ratio = allow / demand if demand > 0 else float("inf")
        margin = ratio / sf
        margins[name] = margin
        if margin < governing[1]:
            governing = (name, margin)
        if margin < 1.0:
            findings.append(finding(
                "error", code,
                f"{name}: {demand:.1f} MPa of {allow:.1f} MPa allowable, "
                f"margin {margin:.2f} < 1.00 at SF {sf}",
                f"params.{'thickness_mm' if name != 'tear-out' else 'edge_mm'}",
            ))

    masses = length * width * thickness * material["rho"]
    score = clamp(governing[1] / 2.0, 0.0, 1.0)
    if governing[1] < 1.0:
        score = min(score, 0.5 * governing[1])

    measures.update({
        "material": material_name,
        "mass_g": round(masses, 2),
        "sigma_net_mpa": round(demands["net-section"][0], 2),
        "sigma_bearing_mpa": round(demands["bearing"][0], 2),
        "tau_tearout_mpa": round(demands["tear-out"][0], 2),
        "margin_net": round(margins["net-section"], 3),
        "margin_bearing": round(margins["bearing"], 3),
        "margin_tearout": round(margins["tear-out"], 3),
        "margin_factor": round(governing[1], 3),
        "governing": governing[0],
        "score_rule": "margin_factor / 2, saturating at 1.0",
    })

    errors = [f for f in findings if f["level"] == "error"]
    if errors:
        score = min(score, 0.49)
    return emit(findings, measures, score if not errors else round(score, 3),
                verdict="fail" if errors else "pass")


def emit(findings, measures, score, verdict=None):
    errors = [f for f in findings if f["level"] == "error"]
    if verdict is None:
        verdict = "fail" if errors else "pass"
    result = {
        "schema": "sovware.dev/verifier-result/v1",
        "verifier": "mech/yield-margin",
        "version": "0.1.0",
        "kind": "numeric",
        "verdict": verdict,
        "score": clamp(score, 0.0, 1.0),
        "findings": findings,
        "measures": measures,
    }
    json.dump(result, sys.stdout)
    sys.stdout.write("\n")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
