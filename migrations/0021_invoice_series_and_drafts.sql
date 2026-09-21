-- The counter is one per series. A number is unique on
-- (party, year, premises, device, ordinal), so the counter that hands the
-- ordinals out is keyed the same way; a second premises or device numbers
-- from 1 on its own, as the law wants.
alter table finance.invoice_numbers
    add column premises text not null default '1',
    add column device   text not null default '1';
alter table finance.invoice_numbers drop constraint invoice_numbers_pkey;
alter table finance.invoice_numbers add primary key (party_id, year, premises, device);

-- A draft that is not wanted is deleted, never cancelled: it was never an
-- invoice and took no number. The rows the old discard left behind
-- (cancelled, no ordinal) go the same way; "cancelled" then always means an
-- issued invoice.
delete from finance.invoices where status = 'cancelled' and ordinal is null;
