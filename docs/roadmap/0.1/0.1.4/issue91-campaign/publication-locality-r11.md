# R11 prospective publication locality repair

G4's remaining profile has2,202 main-thread Init samples: publication includes
564 commit samples,533 insertion samples and298 late-lookup samples; preparation
273; initial lookup331. These are sampled nested scopes, not additive elapsed
fractions. Scratch indexes no longer dominate. SQLite page lookup/pinning,
B-tree insertion and4KiB page writes dominate native encoding.

Historical v0.1.3 consume_checked_owned_page sorts canonical IDs before insertion
and uses cached prepared insertion statements. Packed PreparedAdmission::insert
currently emits locators in pack/producer order and reparses each generated INSERT.
Preserve native FULL/PREFIX physical pack/group/record order and bytes; sort only
the existing bounded locator-reference vector by canonical ObjectId before SQL.
Reuse the connection's existing bounded prepare_cached facility for pack/locator
statements; no cache-size increase or new cache. Keep512/512KiB batches and2MiB/
6MiB ownership ceilings, canonical authentication, late comparisons, rollback,
synchronous publication, and all schema/format/durability contracts.

Regression before repair: a test-only SQLite trigger observes actual locator
insertion order across multiple128-row statements/groups, requiring ascending
canonical IDs. Retain failed order assertion. After repair assert every published
pack BLOB equals the prepared bytes and every canonical object reads back exactly;
run collision/rollback/bounds and affected correctness gates. Existing telemetry
may record sorting time within the public operation, never outside its timer.
Freeze G5 after checks; the same four R6 observations and independent proofs once.

Published namespace100000 has422,065 canonical objects /542,898,158 canonical
bytes: the dramatic elapsed difference is not a several-fold increase in canonical
population. It had130 transactions versus G4's roughly1,165 and a64KiB SQLite
page layout. The accepted4KiB creation policy is a separate storage improvement;
retain it. Do not restore64KiB pages to chase speed or pool its historical costs
with current4KiB costs. Final acceptance still requires storage savings/full157
and all affected matrix members; G4 is not an acceptance baseline.
