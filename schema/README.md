# schemas

Seven JSON Schemas (draft 2020-12). They are normative: `docs/SPEC.md` cites them, the Rust
types in `crates/sov-core` mirror them, and a mismatch is a bug in the code.

| File | Object | Spec |
| --- | --- | --- |
| `program.schema.json` | the artifact the agent mutates | R1, R4, §4.1 |
| `verifier.schema.json` | an oracle, declared before it runs | R2, R6, §4.2 |
| `verifier-result.schema.json` | what a verifier prints on stdout | §4.3 |
| `domain.schema.json` | the gate and the non-goals for a field | R5, §5, §8 |
| `run.schema.json` | one ledger line, hash-chained | R7, §6 |
| `eval.schema.json` | the train / holdout / known-bad corpus | R11, §7, §8 |
| `eval-report.schema.json` | what `sov eval` emits, failures included | R12, §7 |

## rules

- `additionalProperties: false` everywhere. An unknown field is an error, not a forward
  compatibility win. Extensions get a new `schema` string.
- Every hash is `sha256:<64 lowercase hex>` over the exact bytes the field names. The
  canonical form for structured data is compact JSON with sorted keys.
- No schema embeds a vendor, a model, or a URL that must resolve at runtime.

## validate

```sh
python3 - <<'EOF'
import json, glob, sys
try:
    import jsonschema
except ImportError:
    sys.exit("pip install jsonschema")
for f in glob.glob("schema/*.schema.json"):
    jsonschema.Draft202012Validator.check_schema(json.load(open(f)))
    print("ok", f)
EOF
```

The kernel does not depend on a schema library at runtime. It deserializes strictly and
rejects unknown fields, which is the same guarantee for the fields that exist.
