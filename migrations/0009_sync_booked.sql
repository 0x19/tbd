-- A pending row the bank later books is rewritten in place, not duplicated
-- (crates/finance/src/import.rs, `insert_row`). The run that did it should
-- say so: "why did this transaction change" is the audit question a
-- rewritten row raises, and the answer belongs on the row that answers
-- "what did this sync do".
alter table finance.sync_runs add column booked integer not null default 0;
