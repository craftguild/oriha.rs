# Typed native bridge contract

Expose records, enums, options and lists as native language types at renderer/FFI
boundaries. Use JSON only for an explicitly required stable persistence,
transport or external JSON contract. Testing or compatibility alone does not
justify duplicate typed/JSON interfaces for the same operation.

- Use UniFFI `Record` / `Enum` for representable inputs and outputs.
- Pass prepared values, snapshots, patches and exports as typed records, including
  values spanning renderer internal state.
- Export HTML returns only artifact-processing output. It does not return preview
  source maps or patch snapshots.
- Document and test why a new JSON interface needs a JSON protocol instead of
  native types.

[Distribution design](distribution.md#stable-native-interface) records the Rust
namespace requirements that preserve the generated UniFFI ABI.
