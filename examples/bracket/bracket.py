#!/usr/bin/env python3
"""mechanical/lug-bracket source.

The artifact is this program plus the parameters in the sibling program file. The kernel
never executes it (verifier `executes_source` is false); it is the thing a human reads and
a CAD verifier would rebuild. If cadquery is present it writes STEP; if not it prints the
resolved parameter table so the program stays reproducible offline.

    python3 bracket.py lug-bracket.v2.json
"""

import json
import sys

DEFAULT = {
    "length_mm": 120.0,
    "width_mm": 40.0,
    "thickness_mm": 4.0,
    "hole_d_mm": 6.6,
    "hole_count": 2,
    "edge_mm": 12.0,
    "hole_pitch_mm": 80.0,
    "fillet_mm": 4.0,
    "material": "6061-T6",
}


def load(path):
    with open(path, "r", encoding="utf-8") as fh:
        program = json.load(fh)
    params = dict(DEFAULT)
    for key, value in (program.get("params") or {}).items():
        params[key] = value.get("value") if isinstance(value, dict) else value
    return params


def build(params):
    """Two-hole flat lug. Geometry only: the checks live in the verifier."""
    import cadquery as cq

    length = float(params["length_mm"])
    width = float(params["width_mm"])
    thickness = float(params["thickness_mm"])
    hole_d = float(params["hole_d_mm"])
    holes = int(params["hole_count"])
    pitch = float(params["hole_pitch_mm"])
    edge = float(params["edge_mm"])
    fillet = float(params["fillet_mm"])

    plate = (
        cq.Workplane("XY")
        .box(length, width, thickness, centered=(True, True, False))
        .edges("|Z")
        .fillet(fillet)
    )

    # Holes on the long axis, first one `edge` from the end.
    first_x = -length / 2 + edge + hole_d / 2
    positions = [(first_x + i * pitch, 0.0) for i in range(holes)]
    plate = (
        plate.faces(">Z")
        .workplane()
        .pushPoints(positions)
        .hole(hole_d)
    )
    return plate


def main():
    program_path = sys.argv[1] if len(sys.argv) > 1 else None
    params = load(program_path) if program_path else dict(DEFAULT)

    try:
        part = build(params)
    except ImportError:
        for key in sorted(params):
            print(f"{key} = {params[key]}")
        return 0

    out = program_path.rsplit(".", 1)[0] + ".step" if program_path else "bracket.step"
    part.val().exportStep(out)
    print(out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
