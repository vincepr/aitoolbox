# Storage Options

Storage should be selected per subsystem rather than forced into one model too early.

## Working Assumptions

- human-authored configuration can live comfortably in text files
- structured and query-heavy state may justify SQLite
- some services or tools may eventually need their own local stores
- caches should be reconstructable where practical

## Non-Decision

This repository does not currently assume:

- one global database
- multiple mandatory databases
- SQLite for every structured need

The architectural goal is to keep storage replaceable behind clear subsystem boundaries.
