# Architecture documentation

Design docs for KB2 features live here, split into two levels. **Non-trivial features —
anything that adds infrastructure, a new async flow, a schema change, or a cross-crate
contract — should have both.** Small, self-contained changes don't need a design doc.

## Directory layout

| Directory | Audience | Answers |
|---|---|---|
| `high-level-architecture/` | Reviewers, future maintainers, non-authors | *What* problem, *why* this shape, what was decided and what was rejected |
| `low-level-architecture/` | The implementer | *How* — schemas, code snippets, message contracts, infra diffs, tuning, failure modes |

## Naming — one feature, one name, both levels

Each feature uses the **same kebab-case filename** in both directories. The directory
communicates the level; the filename communicates the feature. Never encode the level in
the filename (no `-low-level`, `-hld`, `-v2` suffixes).

```
docs/
  high-level-architecture/
    verify-role-reconciliation.md
  low-level-architecture/
    verify-role-reconciliation.md   ← same name, deeper detail
```

Pick the filename from the feature/capability, not the ticket or component
(`verify-role-reconciliation`, not `kb2-142` or `consumer-changes`). Keep it stable once
merged — renaming breaks inbound links.

## Every doc starts with

- A one-line **Status**: `Proposed` | `Accepted` | `Implemented` | `Superseded`.
- A **companion link** to its counterpart at the other level (relative path, as the
  existing pair does). A high-level doc without a low-level companion is fine while a
  feature is still just proposed; a low-level doc should always link back up.

## When you add or change a doc

1. Create/update **both levels** for a non-trivial feature — a high-level rationale with
   no implementation detail tends to rot, and a low-level spec with no "why" gets
   second-guessed in review.
2. Cross-link the two files at the top so either can be found from the other.
3. When a feature ships, flip the Status to `Implemented` rather than deleting the doc —
   it becomes the record of *why* the code looks the way it does.
4. If a later design replaces an older one, mark the old doc `Superseded` and link forward
   to the replacement instead of editing history out.

## What each level should contain

**High-level** — problem statement and its scale/impact; the decisions taken (ideally a
short decision record, as `verify-role-reconciliation.md` does); a diagram of the target
flow; throughput/cost characteristics; a rollout outline; and **alternatives considered
with the reason each was rejected** (this is the part reviewers most often want and
authors most often omit).

**Low-level** — a change map (files touched and how); concrete schemas/migrations; code
snippets in the repo's actual conventions; message/API contracts; the infra diff; tuning
constants with rationale; a failure matrix; a testing plan; and an ordered, independently
shippable implementation sequence.

## Reference example

`verify-role-reconciliation.md` (both levels) is the worked example to mirror for
structure and depth.
