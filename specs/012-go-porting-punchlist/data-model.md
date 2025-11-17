# Data Model: Go Porting Punchlist

## Overview
Represents parity gaps between Rust reference and Go port, associated remediation tasks, and closure reporting.

## Entities

### GapItem
- id: string (format `GAP-###`)
- category: enum { Functional, Data, Observability, Operability }
- priority: enum { Critical, High, Medium, Low }
- expected_behavior: string
- actual_behavior: string
- reproduction_steps: string
- acceptance_criteria: string
- status: enum { Open, InProgress, Closed, Deferred }
- tasks: array[RemediationTask.id]
- created_at: timestamp
- updated_at: timestamp

Validation:
- id unique; priority/category required; status transitions must follow: Open -> InProgress -> Closed | Deferred; Deferred cannot return to Open without new id.

### RemediationTask
- id: string (format `TASK-###`)
- gap_ids: array[GapItem.id]
- title: string
- description: string
- acceptance_criteria: string
- estimate: integer (story points or hours)
- status: enum { Pending, Active, Done, Blocked }
- created_at: timestamp
- updated_at: timestamp

Validation:
- Must reference at least one GapItem; status transitions: Pending -> Active -> Done | Blocked; Blocked -> Active allowed.

### ParityReport
- id: string (format `REPORT-YYYYMMDD`)
- generated_at: timestamp
- total_gaps: integer
- closed_gaps: integer
- open_gaps: integer
- deferred_gaps: integer
- critical_remaining: integer
- summary: string
- notes: string

Validation:
- critical_remaining must be 0 for closure milestone.

## Relationships
- GapItem many-to-many RemediationTask (tasks field in GapItem, gap_ids in RemediationTask)
- ParityReport snapshots aggregate counts; does not own GapItems.

## State Transitions Summary
```
GapItem: Open -> InProgress -> Closed
                          \-> Deferred
RemediationTask: Pending -> Active -> Done
                          \-> Blocked -> Active
ParityReport: Generated (immutable)
```

## Derived Fields
- ParityReport.closed_gaps = count(GapItem.status == Closed)
- ParityReport.open_gaps = count(GapItem.status in {Open, InProgress})
- ParityReport.deferred_gaps = count(GapItem.status == Deferred)
- ParityReport.critical_remaining = count(GapItem.priority == Critical && status != Closed)

## Notes
No persistence schema changes required now; data can exist as documentation artifacts (YAML/JSON). Future enhancement could store in DB.
