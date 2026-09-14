# Architecture contract template

Add this section to `work-items/<card-id>.md`, after "Flow and delivery plan". Delete lines that do not
apply, and for each deletion write "not applicable: <reason>". A silently missing line reads as
forgotten.

```markdown
## Architecture contract

- Author and date: <stage agent or person>, YYYY-MM-DD
- Route: full | discovery | expedite
- Brief sections relied on: §…
- ADRs: ADR-NNNN (<status>), …

### Boundaries and placement

| Acceptance criterion | Boundaries crossed | Crate or module | New or existing |
|---|---|---|---|
| AC1 | CLI adapter → application → domain | `crates/application::evaluate` | existing |

### Crates and dependency direction

- Crates touched, and the boundary each represents:
- New crates, and why a module is not enough (K §10.2):
- Allowed dependencies (for example `domain` → none; `application` → `domain`; adapters → `application`, `domain`):
- Dependencies added, with version, licence, the source checked, and the S §20.5 answers:

### Ports

| Port | Operation signatures | Send and dispatch choice | Error type and variants | Transaction or ordering needs |
|---|---|---|---|---|

### Data and translation

- Types introduced, their invariants, and their validating constructors:
- Translations at each boundary (DTO → command → domain; row ↔ entity):
- Stored or emitted shapes, with the contract direction and compatibility promise (brief §7):
- Migration steps (expand and contract), if any:

### Runtime behavior

- Configuration keys (prefix and separator from brief §12), with defaults and validation:
- Concurrency model, the blocking-work strategy, cancellation safety:
- Bounds on input size, depth, time, allocation, and queues, with the overload behavior:
- Timeouts, deadlines, retries, and idempotency for each external call:
- Observability: spans and fields, metrics and their labels, domain outcomes:
- Startup and shutdown effects:

### Security

- Trust boundaries crossed, and where authorization is enforced (K §22.2):
- Untrusted inputs, with their bounds and validation points:
- Secrets and redaction:
- Abuse cases the tests must cover:

### Test plan

| Acceptance criterion | Layer | Test (planned path::name) | Real dependency or fake | Negative cases |
|---|---|---|---|---|

- Property, fuzz, performance, or resilience checks, and which stage runs them:

### Constraints for the implementer

- Must:
- Must not:
- Stop and ask if:

### Left unverified

- <what>, verified by <stage or card>, because <reason>
```
