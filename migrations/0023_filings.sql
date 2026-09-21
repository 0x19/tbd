-- An ePorezna form (PD, PDV, PDV-S, ZP, JOPPD, PD-IPO, TZ) uploaded as XML: the
-- document holds the bytes, this row what the form says. `values` is a flat map
-- of body path to the decimal string as filed ("1" -> "177733.47" for PD
-- Podatak1, "200.Vrijednost" for PDV, "A.PredujamPoreza.P1" for JOPPD page A);
-- `rows` holds the repeated blocks (JOPPD page B recipients, ZP supplies, PD
-- donation recipients, PD-IPO persons) as an array of objects, each with a
-- `_kind` naming its container. Amounts stay strings so nothing is rounded;
-- the code converts to minor units exactly when it needs a number.
create table finance.filings (
    document_id    uuid        primary key references finance.documents(id) on delete cascade,
    party_id       uuid        not null references public.parties(id) on delete cascade,
    form           text        not null check (form in ('pd', 'pdv', 'pdv_s', 'zp', 'joppd', 'pd_ipo', 'tz')),
    -- Metapodaci/Uskladjenost, e.g. ObrazacPD-v9-0; built from the root
    -- namespace when the form does not say.
    schema         text        not null,
    period_from    date        null,
    period_to      date        null,
    -- Zaglavlje/Obveznik/OIB, or StranaA/PodnositeljIzvjesca/OIB for JOPPD.
    -- Equals the party's org OIB: the upload refuses anything else.
    oib            text        not null check (oib ~ '^[0-9]{11}$'),
    obveznik       text        not null default '',
    -- Metapodaci/Datum: when the software prepared the form. Not the time
    -- ePorezna accepted it; the XML does not carry that.
    prepared_at    timestamptz null,
    author         text        not null default '',
    -- Metapodaci/Identifikator, the preparing software's id for the form.
    identifier     text        not null default '',
    -- JOPPD StranaA/OznakaIzvjesca (yyDDD); empty for other forms.
    report_mark    text        not null default '',
    values         jsonb       not null default '{}'::jsonb,
    rows           jsonb       not null default '[]'::jsonb,
    parsed_at      timestamptz not null default clock_timestamp(),
    parser_version text        not null,
    -- Set when the form was recognised but the body could not be read in
    -- full; `values` may be partial. A re-read after a parser upgrade clears it.
    error          text        null
);
create index filings_party_form_period_idx on finance.filings (party_id, form, period_from desc nulls last);
create index filings_party_identifier_idx on finance.filings (party_id, identifier) where identifier <> '';

alter table finance.filings enable row level security;
create policy filings_visible on finance.filings using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));

grant select, insert, update, delete on finance.filings to tbd_finance;
