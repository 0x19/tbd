-- A person pressing Refresh from the UI is an attended access under PSD2:
-- the call carries their address and the bank does not count it against the
-- four-a-day. The run row says which kind it was.
alter table finance.sync_runs drop constraint sync_runs_trigger_check;
alter table finance.sync_runs add constraint sync_runs_trigger_check
    check (trigger in ('scheduled', 'manual', 'attended', 'initial'));
