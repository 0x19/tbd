-- A delivery is the invoice going out, or a reminder that it is still owed.
alter table finance.invoice_deliveries
    add column kind text not null default 'invoice' check (kind in ('invoice', 'reminder'));
