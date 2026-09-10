# Security policy

## Supported versions

Security fixes are applied to the latest version on the `main` branch.

| Version | Supported |
| --- | --- |
| 2.0.0-alpha.x | Yes |
| 1.0.2 legacy browser build | Yes |
| Earlier versions | No |

The native-only source tree retains Backup and desktop-transition import
compatibility. Browser migration support does not mean that this checkout
contains a browser frontend or can rebuild a legacy deployment.

## Security boundaries

Notes, snapshots, and exported Backups are local files without application-level
encryption. Native Preview disables scripts and blocks remote resources.
User-activated web and email links pass to the system handler. See
[PRIVACY.md](PRIVACY.md) for the data and network boundary.

## Reporting a vulnerability

Please report suspected vulnerabilities privately through
[GitHub Security Advisories](https://github.com/astrazds/nota/security/advisories/new).
Include the affected version, impact, reproduction steps, and any suggested
mitigation. Please do not open a public issue for an undisclosed vulnerability.

Include the package type and desktop environment for native issues. Use a
minimal synthetic Note or export to reproduce the problem. Do not attach your
live collection, private Backups, or corrupt-payload quarantine files to a
public report.
