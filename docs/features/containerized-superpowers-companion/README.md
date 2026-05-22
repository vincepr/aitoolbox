# Containerized Superpowers Companion

## Summary

Add complementary skills and supporting conventions that allow container-isolated development workflows to work alongside `obra/superpowers` without forking its full methodology.

## Problem

`obra/superpowers` is strongly oriented around isolated work via worktrees and branch completion flows. Its core execution and review methodology is still useful, but teams may want task execution to happen inside containers instead of directly in a worktree on the host machine.

This feature is about adding a companion layer that:

- preserves the useful parts of the Superpowers workflow
- introduces container-specific setup and cleanup skills
- makes execution mode explicit
- avoids rewriting unrelated skills such as brainstorming, TDD, or code review

## Goal

Define an interoperable set of local skills, docs, and conventions for containerized development sessions that can be used together with Superpowers-style planning and execution.

## Non-Goals

- replacing `obra/superpowers`
- upstreaming these skills into Superpowers core
- building a full container orchestration platform in the first iteration
- solving remote CI or deployment concerns

## Expected Outputs

- feature-specific documentation
- companion skills for container workspace lifecycle
- conventions for how plans and execution handoff should refer to containerized work
- clear boundaries between host-side coordination and container-side implementation
