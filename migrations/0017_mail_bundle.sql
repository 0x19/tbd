-- A mail may carry the accountant's bundle: one zip built by the service from
-- the receipts and the summary files the page prepared. The zip is not a
-- document (its receipts are, and stay linked through mail_documents); the
-- row keeps its file name so the record says what went out.
alter table finance.mails add column bundle text null;
