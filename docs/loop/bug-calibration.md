# Bug-hunter calibration

Findings the lead rejected as irrelevant to brain-swap. The bug hunter reads this before probing and never reports the same kind of finding again. The lead appends one entry per rejected finding; the owner prunes or overrules entries.

Entry format: task, the finding in one line, why it is irrelevant, the general rule it teaches.

## E1-F1-T4 Stamps (2026-10-10, rejected by the owner)

- Finding: an RFC 3339 stamp with more than 9 fraction digits (`2026-10-08T10:31:05.1234567890+03:00`) parses to `None`, because jiff holds nanoseconds.
- Why irrelevant: brain-swap writes stamps without a fraction, and nobody types ten fraction digits into a timeline heading. The outcome is the defined graceful one (age `?`). It blocked a correct task for a night and stalled the whole loop.
- Rule: precision and range limits of a library or standard, reached only by machine-generated edge values, are not findings.
