# Security Policy

## Supported versions

Tasklynx is pre-1.0 software. Security fixes are applied to the latest published release and to `main`.

| Version | Supported |
| --- | --- |
| `0.0.1-beta.x` (latest) | Yes |
| Older releases | No |

## Reporting a vulnerability

Please **do not** report security vulnerabilities through public GitHub issues, discussions, or pull requests.

Report privately to [peekstore585@gmail.com](mailto:peekstore585@gmail.com) with:

- a description of the issue and its impact,
- steps to reproduce or a proof of concept,
- the affected version (`tx --version`) and platform,
- any suggested remediation.

You can also use GitHub's private vulnerability reporting on the repository if it is enabled.

### What to expect

- We aim to acknowledge reports within 72 hours.
- We will investigate, keep you informed of progress, and coordinate a disclosure timeline with you.
- We will credit you in the release notes unless you prefer to remain anonymous.

Please give us a reasonable opportunity to address the issue before public disclosure.

## Scope

Tasklynx is a local-first application. The relevant attack surface is:

- the local data store (`tasks.json`) and config (`config.toml`),
- the optional web server (`tx web`),
- the installers (`installer.sh`, `installer.ps1`) and release artifacts.

### Web server

`tx web` binds to `127.0.0.1` by default. It has **no authentication**. Do not expose it to untrusted networks. If you need remote access, place it behind an authenticated reverse proxy or use a private tunnel, and treat anyone who can reach the port as able to read and modify your tasks.

### Out of scope

- Vulnerabilities in operating systems, terminals, or third-party dependencies that are already fixed upstream (please report those upstream).
- Issues that require an attacker to already have local access to your user account and its files.
- Social-engineering attacks against maintainers or contributors.

## Dependency and supply-chain security

- Dependencies are pinned in `Cargo.lock`.
- Release binaries are built by GitHub Actions from tagged source.
- Checksums are published with each release (`SHA256SUMS.txt`).

Verify a download before running it:

```bash
sha256sum -c SHA256SUMS.txt
```

## Safe configuration practices

- Keep the default `host = "127.0.0.1"` unless you have added access control.
- Do not commit your `tasks.json` or `config.toml` to a public repository if they contain sensitive information.
- Install only from official releases or `github.com/peekstore/tasklynx`.
