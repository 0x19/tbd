-- The client a company bills by default: the one the invoices list opens on
-- and a new draft is for. One per company, chosen on the Clients page.
alter table finance.clients add column is_default boolean not null default false;
create unique index clients_default_idx on finance.clients (party_id) where is_default;
