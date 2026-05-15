# RCTX Fixtures

These fixtures exercise the minimal shared context package boundary. RCTX owns
canonical unit identity, graph identity, source refs, and crosswalk records. It
does not own vote totals, district assignments, rendered maps, or historical
lineage.

`l0-shared-context` is a tiny positive fixture with two synthetic precinct
units, one graph identity record, one identity crosswalk, package hashes, and a
verify transcript. Negative coverage lives in `rctx-core` fixture helpers for
missing source refs and bad crosswalk weights.
