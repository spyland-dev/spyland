<div align="center">

**English** | [Русский](/docs/ru/SECURITY.md)

</div>

# Security Policy

## Reporting a Vulnerability

If you believe you have discovered a security vulnerability, privacy issue, or any behavior
that could compromise user data, please report it privately.

Please do **not create a public issue** for vulnerabilities that may expose users to risk.

Preferred reporting methods:

* GitHub Private Vulnerability Reporting
* Direct contact with me via GitHub or via my email.

When submitting a report, please include:

* A description of the issue
* Steps to reproduce the issue
* The affected version(s)
* Relevant logs, screenshots, or proof-of-concept code if available
* Potential impact of the vulnerability

All reports will be reviewed in good faith.

## Supported Versions

As the project is under active development and has not yet reached a stable 1.0 release, only the
latest development version is considered supported for security fixes.

Older versions may not receive security updates.

## Project Security Model
The security model of `spyland` is based on the following principles:

- ### Local-first
Activity data is intended to remain on the user's device.

- ### No Built-in Telemetry
`spyland` does not intentionally transmit user activity data to external services.

- ### Open Source
All source code is publicly available for inspection and auditing.

- ### Explicit Components
Activity collection is performed by separately installed backends chosen by the user.

- ### User Control
Users are expected to control which backends are installed and running on their systems.

## Threat Model

`spyland` is not intended to protect against:
* Fully compromised operating systems
* Malware running with the same privileges as the user
* Physical attackers with unrestricted access to a running and unlocked session
* Database leaks not caused by the project or its official component
* Malicious third-party clients, utilities, or backends

## Acknowledgements
Security reports, audits, reviews, and responsible disclosure efforts are greatly appreciated and
help improve the project for all users.
