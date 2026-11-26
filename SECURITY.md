# Security Policy

## Supported versions

Only the latest main branch is currently supported. There are no stable releases yet.

## Reporting a vulnerability

Do not publish an exploitable vulnerability in a public issue. Use GitHub private vulnerability reporting when available. If private reporting is unavailable, contact the repository owner through the email address listed on the GitHub profile and include the impact, affected commit/WebKit revision, reproduction steps, and proposed mitigation.

Do not include passwords, private browsing data, session cookies, or real user information in reports.

## Security boundaries

- Web content is untrusted input.
- Never disable TLS certificate validation to make a site load.
- Never mix WebKit DLLs and helper processes from different builds.
- Do not log form contents, cookies, passwords, or private browsing URLs.
- Keep WebKit and bundled dependencies updated and preserve upstream license/security notices.
