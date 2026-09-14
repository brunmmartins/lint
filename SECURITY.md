# Security policy

## Reporting a vulnerability

Report privately through GitHub's private vulnerability reporting for brunmmartins/lint, once the
maintainer has created the repository and enabled it in the security settings. Until then, contact the
maintainer privately through GitHub. Never report a vulnerability in a public issue.

Do not describe an unfixed vulnerability in a public issue, a branch name, or a commit message. Those are
replicated widely and kept indefinitely (S §7.5).

## Handling

| | |
|---|---|
| Triage owner | The maintainer |
| Acknowledgement and remediation service levels | None promised: a local tool with no users beyond the maintainer (brief §15) |
| Supported versions | None released yet |

- Security findings go on the board with restricted access. Exploit details stay out of public card
  fields (K §22.3).
- Expedite handling applies only when the expedite criteria in [docs/workflow-policy.md](docs/workflow-policy.md)
  are met. A scanner score alone does not qualify (K §7.3, §16.4).
- A committed secret is rotated first. Removing it from history does not make it safe again (S §7.20).
