-- What a client is usually billed for, so a draft starts from the contract
-- and not from a blank page.
--
-- Thirty issued invoices say the shape: one row that never changes (the
-- retainer), one whose price changes every month (on-call), and now and then
-- a third (a bonus, a credit). Three modes, then:
--
--   fixed     included on every draft with its price
--   variable  included on every draft; the price is asked for each time,
--             pre-filled from the last invoice that carried the same row
--   optional  offered in the editor, not included until chosen
--
-- The draft still records `prefilled_from`; the templates decide which rows
-- exist, the last invoice decides what a variable row cost last time.

create table finance.line_templates (
    id               uuid        primary key,
    client_id        uuid        not null references finance.clients(id) on delete cascade,
    position         integer     not null,
    description      text        not null check (length(description) between 1 and 500),
    mode             text        not null check (mode in ('fixed', 'variable', 'optional')),
    quantity_milli   bigint      not null default 1000 check (quantity_milli > 0),
    unit_price_minor bigint      not null default 0,
    enabled          boolean     not null default true,
    created_at       timestamptz not null default clock_timestamp(),
    updated_at       timestamptz not null default clock_timestamp()
);
create index line_templates_client_idx on finance.line_templates (client_id, position) where enabled;

-- Which template a line came from, so a variable row can find its last price
-- and the editor can show what is still on offer. Null for a line typed by
-- hand.
alter table finance.invoice_lines
    add column template_id uuid null references finance.line_templates(id) on delete set null;

grant select, insert, update, delete on finance.line_templates to tbd_finance;
