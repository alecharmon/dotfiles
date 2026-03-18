---
name: save-docs
description: Use when creating or saving design documents, architecture docs, plans, or any documentation artifacts during skills or creative workflows. Triggers on doc/design file creation.
---

# Save Docs

## Overview

All design, architecture, and planning documents go in `~/dev/docs/` as a flat directory. An INDEX.md tracks every doc for future discovery.

## When to Use

- Creating design docs, architecture docs, or plans
- Any skill or workflow that produces documentation artifacts
- When user asks to save or write a doc/design file

## Process

1. **Determine project name** from the current working directory:
   - If cwd is under `~/dev/<project>/...`, use `<project>`
   - If cwd IS `~/dev/<project>`, use `<project>`
   - Otherwise, use the basename of cwd

2. **Create the doc** at `~/dev/docs/<date>-<project>-<slug>.md`
   - Date format: `YYYY-MM-DD`
   - Slug: lowercase, hyphenated summary (e.g. `auth-service-design`)
   - Example: `2026-03-18-fern-platform-auth-service-design.md`

3. **Update the index** at `~/dev/docs/INDEX.md`

## Index Format

`~/dev/docs/INDEX.md` is a table of all docs:

```markdown
# Docs

| Date | Project | File | Description |
|------|---------|------|-------------|
| 2026-03-18 | fern-platform | 2026-03-18-fern-platform-auth-service-design.md | Design for new auth service with OAuth2 flows |
| 2026-03-15 | dotfiles | 2026-03-15-dotfiles-api-migration-plan.md | Migration plan from REST to GraphQL |
```

- One row per doc, newest first
- Description: 1 sentence summarizing the doc's purpose
- If INDEX.md doesn't exist, create it with the header and first entry
- If it exists, prepend the new row after the header

## Common Mistakes

- Putting docs in the repo itself instead of `~/dev/docs/`
- Forgetting to update INDEX.md after creating a doc
- Creating subdirectories — keep it flat
