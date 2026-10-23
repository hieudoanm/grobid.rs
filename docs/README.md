# grobid.rs documentation

This directory contains the project documentation for **grobid.rs**, a
local-first tool that converts PDF articles to Markdown via GROBID TEI XML.
Files are prefixed with a number to suggest a reading order.

## Reading order

| #   | File                                                     | Description                              |
| --- | -------------------------------------------------------- | ---------------------------------------- |
| 01  | [README.md](./README.md)                                 | Docs index (this page)                   |
| 02  | [01-architecture.md](./01-architecture.md)               | Repository layout, crates, data flow     |
| 03  | [02-cli.md](./02-cli.md)                                 | Command-line interface reference         |
| 04  | [03-grobid-service.md](./03-grobid-service.md)           | GROBID lifecycle, Docker, endpoints      |
| 05  | [04-tei-to-markdown.md](./04-tei-to-markdown.md)         | TEI parsing, metadata, Markdown rendering |
| 06  | [05-development.md](./05-development.md)                 | Local development commands and fixtures  |
| 07  | [06-release.md](./06-release.md)                         | CI, releases, and install scripts        |

## Related

- Project README: `../README.md`
- Agent/contributor guide: `../AGENTS.md`
- Rust crates: `../crates/` (`grobid-core`, `grobid-cli`)
- Landing page: `../packages/landing/`
- CI workflow: `../.github/workflows/ci.yaml`
