# Validation Principles

The repository should prefer explicit failure over silent degradation.

## Expectations

- parse errors should fail immediately
- missing references should be surfaced clearly
- unsupported execution modes should fail loudly
- renamed or removed config targets should not be ignored
- optionality should exist only where behavior is genuinely optional

## Validation Layers

- shape validation
- reference validation
- environment validation
- execution validation

## Operational Rule

Warnings are acceptable for advisory information. Anything that changes whether the system can be trusted should be treated as an error.
