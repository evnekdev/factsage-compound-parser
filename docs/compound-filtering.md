# Bounded compound filtering

`CompoundFilterPlan` selects complete ID-1 compound groups by source encounter
index. It preserves the selected group's phases, Cp ranges, transitions, ID-11
records, comments and unknown chunks in original physical order and with exact
native bytes. The ID-9 header is always retained. An empty selection emits a
header-only database.

The plan is bound to one raw stream generation. It rejects invalid indexes,
duplicate selection and selected groups with duplicate phase IDs, orphan Cp or
ID-11 ranges, or ambiguous phase links. Materialization serializes, strict
reparses and builds a domain index. Byte comparison handles preserved NaN
payloads, which typed floating-point equality would misclassify.

This is a whole-group native output capability. Selecting only some phases in a
compound requires an evidenced CDB counter/reference policy and explicit
handling of group comments and opaque records. The current API does not delete
such dependencies or synthesize new CDB IDs. The Database Compare output adapter
still needs to bind semantic selections to this provider plan and report
partial-group requests as a typed pending capability.
