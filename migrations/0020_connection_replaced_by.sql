-- A renewed consent takes the accounts over; the consent it replaces is
-- revoked and points at it, so the page can say "replaced by a newer consent"
-- instead of leaving an authorized row with no accounts.
alter table finance.connections add column replaced_by uuid null references finance.connections(id) on delete set null;
