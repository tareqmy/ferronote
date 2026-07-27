---
name: Ferronote Git Conventions
description: Use this skill when committing changes or managing git branches in the Ferronote project.
---

# Ferronote Git Conventions

Ferronote strictly adheres to the Conventional Commits specification. Ensure your commits follow this structure:

## Commit Messages
Structure: `<type>: <short-description>`

Types:
- `feat:` for new features
- `fix:` for bug fixes
- `refactor:` for non-functional changes (no new features or bug fixes)
- `docs:` for documentation
- `test:` for test additions/changes
- `chore:` for tooling, CI, dependencies

## Branch Naming
When creating branches, name them according to their purpose:
- `feat/short-description`
- `fix/short-description`
