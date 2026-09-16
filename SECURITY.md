# Security policy

## Supported versions

`lint` is currently pre-release. Security fixes are applied to the latest code
on `main`; no released version has a separate support commitment yet.

## Reporting a vulnerability

Use GitHub's private vulnerability reporting for `brunmmartins/lint`. If private
reporting is not available, contact the maintainer privately through GitHub.

Do not report an unfixed vulnerability in a public issue, branch name, commit
message, or review comment. Include the affected version or commit,
reproduction steps, impact, and any suggested mitigation in the private report.

The maintainer triages reports and will coordinate disclosure after a fix is
available. No fixed response or remediation time is currently promised.

If a secret is committed, rotate it immediately. Removing it from Git history
does not make the exposed secret safe again.
