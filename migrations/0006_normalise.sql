-- Matching counterparty names without tripping over diacritics.
--
-- The real data has the same payee spelled both ways: `DRŽAVNI PRORAČUN
-- REPUBLIKE HRVATSKE` and `DRZAVNI PRORACUN REPUBLIKE HRVATSKE`, in the same
-- account, weeks apart. A rule written against either spelling silently misses
-- half the rows, and the money quietly lands in "uncategorised" rather than
-- anywhere a person would notice.
--
-- `unaccent` would do this, but it is an extension: it needs superuser to
-- install and is not guaranteed on a managed Postgres. `translate` needs
-- nothing, covers the Croatian set exactly, and is immutable -- which is what
-- lets it be indexed later.

create or replace function finance.normalise(value text)
returns text
language sql
immutable
parallel safe
returns null on null input
as $$
    select upper(translate(
        value,
        'ČčĆćĐđŠšŽžÀÁÂÃÄÅàáâãäåÈÉÊËèéêëÌÍÎÏìíîïÒÓÔÕÖòóôõöÙÚÛÜùúûü',
        'CcCcDdSsZzAAAAAAaaaaaaEEEEeeeeIIIIiiiiOOOOOoooooUUUUuuuu'
    ))
$$;

comment on function finance.normalise(text) is
    'Uppercase and strip Croatian diacritics, so a rule matches a payee however the bank spelled it.';

-- Rules and aliases both match on the normalised form from here.
create index bank_transactions_counterparty_normalised_idx
    on finance.bank_transactions (finance.normalise(counterparty_name))
    where counterparty_name is not null;

-- The view resolves aliases against the same normalisation the rules use, so a
-- name recognised by one is recognised by the other.
create or replace view finance.transactions_enriched as
select
    t.*,
    exists (
        select 1 from finance.accounts a
         where a.iban is not null
           and a.iban = t.counterparty_iban
    ) as internal,
    coalesce(al.canonical, t.counterparty_name) as counterparty_canonical
from finance.bank_transactions t
left join finance.counterparty_aliases al
       on al.party_id = t.party_id
      and (
        (al.exact and finance.normalise(coalesce(t.counterparty_name, '')) = al.match_normalised)
        or (not al.exact
            and finance.normalise(coalesce(t.counterparty_name, ''))
                like '%' || al.match_normalised || '%')
      );

grant execute on function finance.normalise(text) to tbd_finance;
grant select on finance.transactions_enriched to tbd_finance;
