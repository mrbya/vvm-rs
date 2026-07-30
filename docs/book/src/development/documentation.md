# Documentation

## Source Of Truth Policy

| Information | Authoritative source |
| --- | --- |
| Project overview | root `README.md` |
| Workflow tutorials | this book |
| Exact API semantics | rustdoc |
| Package introduction | package README |
| Runnable example procedure | local example README |
| Supported versions | compatibility docs plus CI |
| Persisted coverage schema | `docs/coverage-json-v1.md` |
| Contributor commands | `justfile` and contributor docs |
| Roadmap and work tracking | Backlog.md |

When a feature changes, update every surface that owns part of that contract.
For example:

- a new environment variable needs rustdoc, the reference chapter, and any
  affected example or workflow docs;
- a new public macro needs rustdoc, API-guide coverage, and trybuild tests;
- a compatibility change needs CI evidence and the compatibility chapter to move
  together.
