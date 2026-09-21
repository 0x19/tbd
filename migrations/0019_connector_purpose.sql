-- A connector's purpose. A link is asked for reading (pulling receipts),
-- sending (mail as the account, nothing read), or both. The pending row
-- keeps the purpose so the callback can honour it, and the linked row
-- records what the provider granted *within* it: a send-only link records
-- can_read = false even when the provider's grant could read. Every row so
-- far was linked with the read consent.
alter table finance.connectors add column purpose text not null default 'both';
alter table finance.connectors add column can_read boolean not null default true;
