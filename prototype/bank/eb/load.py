"""Flatten data/raw/transactions-*.json into data/bank.sqlite.

    python3 -m eb.load

One wide table, no normalisation, no cleverness. The point is to be able to ask
questions in SQL an hour after pulling, not to model anything — the real schema
lives in the plan.

Amounts arrive as strings ("6.22"); they are stored as REAL for arithmetic and
kept verbatim in amount_raw so nothing is lost to float.
"""

import glob
import json
import pathlib
import re
import sqlite3

DB = pathlib.Path(__file__).resolve().parent.parent / "data" / "bank.sqlite"
RAW = pathlib.Path(__file__).resolve().parent.parent / "data" / "raw"

DDL = """
drop table if exists tx;
create table tx (
    entry_reference text primary key,
    profile         text,      -- business | personal
    account_uid     text,
    iban            text,
    currency        text,
    amount          real,      -- signed: negative is money out
    amount_raw      text,
    credit_debit    text,
    status          text,
    booking_date    text,
    value_date      text,
    counterparty    text,      -- creditor on a debit, debtor on a credit
    counterparty_iban text,
    reference_number  text,    -- the structured poziv na broj, model included
    remittance      text,      -- remittance_information joined with ' | '
    note            text,
    mcc             text,
    raw             text
);
create index tx_date on tx (booking_date);
create index tx_cp   on tx (counterparty);
"""


def _accounts() -> dict[str, tuple[str, str, str]]:
    """uid -> (profile, iban, currency), from the sessions recorded at link time."""
    out = {}
    for path in (RAW.parent / "sessions").glob("*.json"):
        profile = path.stem
        for account in json.loads(path.read_text()).get("accounts") or []:
            if isinstance(account, dict):
                out[account["uid"]] = (
                    profile,
                    (account.get("account_id") or {}).get("iban") or "",
                    account.get("currency") or "",
                )
    return out


def _uid_of(filename: str, accounts: dict) -> str:
    """transactions-<profile>-<index>[-<currency>]-p<n>.json -> the uid."""
    m = re.match(r"transactions-(\w+)-(\d+)", pathlib.Path(filename).name)
    if not m:
        return ""
    profile, index = m.group(1), int(m.group(2))
    uids = [u for u, (p, _, _) in accounts.items() if p == profile]
    return uids[index] if index < len(uids) else ""


def main() -> int:
    accounts = _accounts()
    if not accounts:
        print("no sessions found; run eb.link first")
        return 2

    DB.parent.mkdir(parents=True, exist_ok=True)
    db = sqlite3.connect(DB)
    db.executescript(DDL)

    seen, inserted = set(), 0
    for path in sorted(glob.glob(str(RAW / "transactions-*.json"))):
        uid = _uid_of(path, accounts)
        profile, iban, currency = accounts.get(uid, ("?", "", ""))
        for t in json.load(open(path)).get("transactions") or []:
            ref = t.get("entry_reference")
            if not ref or ref in seen:
                continue          # the same page saved under two names must not double-count
            seen.add(ref)
            raw_amount = (t.get("transaction_amount") or {}).get("amount") or "0"
            sign = -1 if t.get("credit_debit_indicator") == "DBIT" else 1
            party = t.get("creditor") if sign < 0 else t.get("debtor")
            party_acct = t.get("creditor_account") if sign < 0 else t.get("debtor_account")
            remittance = t.get("remittance_information") or []
            db.execute(
                "insert into tx values (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                (
                    ref, profile, uid, iban,
                    (t.get("transaction_amount") or {}).get("currency") or currency,
                    sign * float(raw_amount), raw_amount,
                    t.get("credit_debit_indicator"), t.get("status"),
                    t.get("booking_date"), t.get("value_date"),
                    (party or {}).get("name"),
                    ((party_acct or {}).get("iban") or ""),
                    t.get("reference_number"),
                    " | ".join(x for x in remittance if x),
                    t.get("note"), t.get("merchant_category_code"),
                    json.dumps(t, ensure_ascii=False),
                ),
            )
            inserted += 1

    db.commit()
    print(f"{inserted} transactions -> {DB}")
    for profile, count, lo, hi in db.execute(
        "select profile, count(*), min(booking_date), max(booking_date) from tx group by profile"
    ):
        print(f"  {profile:<10} {count:>5} rows  {lo} -> {hi}")
    db.close()
    print("\nQuery it:  sqlite3 data/bank.sqlite  (or python3 -m eb.ask)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
