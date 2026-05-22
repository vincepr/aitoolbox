# Resources

## Primary References

- `obra/superpowers` repository
  - README workflow overview
  - `skills/using-git-worktrees/SKILL.md`
  - `skills/executing-plans/SKILL.md`
  - `skills/subagent-driven-development/SKILL.md`
  - `skills/finishing-a-development-branch/SKILL.md`
  - `skills/writing-plans/SKILL.md`
  - `RELEASE-NOTES.md`

## Why These Matter

- `using-git-worktrees` shows where isolated workspace setup currently lives.
- `finishing-a-development-branch` shows where lifecycle cleanup and branch finalization are tied to worktrees.
- `executing-plans` and `writing-plans` show the explicit workflow references that would need companion guidance.
- `subagent-driven-development` shows that the core subagent/review loop is largely reusable.

## Internal Design Constraints

- local-first developer workflows
- fail-fast validation
- Rust-first implementation for typed tooling
- support for multiple execution strategies over time
- domain-agnostic core

## Open Research Topics

- whether container lifecycle should be driven by one skill or split into setup and completion skills
- whether plans should record execution strategy explicitly
- how host-side Git operations should be coordinated with container-side command execution
- whether container execution should be layered on top of worktrees, or stand alone
- how much of the workflow should rely on Docker versus a broader container runtime abstraction
