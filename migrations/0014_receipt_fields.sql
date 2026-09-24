-- What a receipt says, read out of its text, and what a person corrected.
--
-- `text` is the document's extracted text, for search; `extracted_at` says the
-- reader ran (fields may still be null when it found nothing); `declared_at`
-- says a person set the fields, so a re-read never overwrites them.
alter table finance.documents
    add column invoice_no   text        null,
    add column text         text        null,
    add column extracted_at timestamptz null,
    add column declared_at  timestamptz null;

create index documents_party_date_idx on finance.documents (party_id, doc_date desc nulls last, created_at desc);
