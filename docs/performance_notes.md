# Performance Notes

v4.3 keeps the core workflow in Python.

Reason:

- The current bottleneck is network and provider data fetching, not local text scanning.
- Research routing and interpretation rules are still evolving and should remain easy to edit.
- Introducing Rust or C++ without a measured bottleneck would increase installation and maintenance cost.

## Current Decision

No Rust or C++ sidecar was added in v4.3.

Python remains responsible for:

- orchestration
- OpenBB / yfinance calls
- financial calculations
- asset routing
- report rendering
- AI patch orchestration

## Future Candidate

If local checks become a bottleneck, a Rust sidecar may be useful for:

- report pack validation
- large text scans
- patch numeric-claim linting
- file manifest generation

Any future non-Python helper must include a Python fallback.

