---
name: Ferronote Checks and Linting
description: Use this skill to check code formatting and run the linter before committing changes or whenever evaluating correctness.
---

# Ferronote Checks and Linting

Before completing any task or committing code in the Ferronote project, ensure the code complies with our strict styling and linting rules.

## Formatting
Run the following command to format the code automatically according to project defaults:
```bash
cargo fmt
```

## Linting
Run clippy in pedantic mode and ensure there are no warnings or errors. Address all issues reported by this command:
```bash
cargo clippy -- -W clippy::pedantic
```

Only proceed with commits or consider a feature complete once `cargo clippy` runs cleanly.
