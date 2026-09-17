"""Pull everything the consent allows into data/raw/, untouched.

    python3 -m eb.pull business [months]

Writes raw responses verbatim — these files, not this code, are what the Rust
implementation is built against. Nothing is normalised here on purpose.

Every call spends one of the ASPSP's daily allowance (often 4 per account when the
user is not online), so this pulls once and saves everything.
"""

import datetime as dt
import json
import sys
import time

import requests

from . import config, token

TIMEOUT = 60
DEFAULT_MONTHS = 24
SESSION_DIR = config.RAW_DIR.parent / "sessions"


def _get(path: str, params: dict | None = None) -> tuple[int, dict | str]:
    response = requests.get(
        f"{config.BASE_URL}{path}",
        params=params or {},
        headers=token.auth_header(),
        timeout=TIMEOUT,
    )
    try:
        return response.status_code, response.json()
    except ValueError:
        return response.status_code, response.text


def _save(payload, name: str) -> None:
    config.RAW_DIR.mkdir(parents=True, exist_ok=True)
    path = config.RAW_DIR / f"{name}.json"
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False))
    print(f"    -> {path.name}")


def _rate_limited(status: int, payload) -> bool:
    if status != 429:
        return False
    print("\n  *** 429 RATE LIMITED — this is a finding, recording the body ***")
    print(f"  {json.dumps(payload, ensure_ascii=False)[:600]}")
    _save(payload, "error-429")
    return True


def transactions(uid: str, tag: str, months: int) -> None:
    """Page through the whole window, following continuation_key.

    Erste rejects a continuation key sent on its own:
        422 ParameterValidationException
        "dateFrom in request is not the same as in continuationKey.
         Continuation key is only valid for the same getAccountTransactions parameters"
    So every page repeats the original date range and adds the key.
    """
    since = (dt.date.today() - dt.timedelta(days=30 * months)).isoformat()
    base = {"date_from": since, "date_to": dt.date.today().isoformat()}
    params = dict(base)
    page, total = 0, 0
    while True:
        status, payload = _get(f"/accounts/{uid}/transactions", params)
        if _rate_limited(status, payload):
            return
        if status != 200:
            print(f"    transactions HTTP {status}: {json.dumps(payload)[:400]}")
            _save(payload, f"error-transactions-{tag}-p{page}")
            return
        rows = payload.get("transactions") or []
        total += len(rows)
        _save(payload, f"transactions-{tag}-p{page}")
        cont = payload.get("continuation_key")
        print(f"    page {page}: {len(rows)} rows" + (" (more)" if cont else ""))
        if not cont:
            break
        params = dict(base, continuation_key=cont)
        page += 1
        if page > 50:
            print("    stopping at 50 pages")
            break
        time.sleep(0.4)
    print(f"    {total} transactions over ~{months} months from {since}")


def main() -> int:
    psu_type = sys.argv[1] if len(sys.argv) > 1 else "business"
    months = int(sys.argv[2]) if len(sys.argv) > 2 else DEFAULT_MONTHS

    path = SESSION_DIR / f"{psu_type}.json"
    if not path.exists():
        config.die(f"no session at {path}\n  Run: python3 -m eb.link {psu_type}")
    session = json.loads(path.read_text())

    session_id = session.get("session_id")
    status, live = _get(f"/sessions/{session_id}")
    if status == 200:
        _save(live, f"session-live-{psu_type}")
    else:
        print(f"GET /sessions/{session_id} -> {status}: {json.dumps(live)[:300]}")

    # POST /sessions returns account objects; GET /sessions/{id} returns bare uid
    # strings. Label from the objects recorded at link time.
    accounts = [a for a in (session.get("accounts") or []) if isinstance(a, dict)]
    if not accounts:
        config.die("the session carries no account objects")

    print(f"\n{len(accounts)} account(s) in the {psu_type} session\n")
    for index, account in enumerate(accounts):
        uid = account.get("uid")
        iban = (account.get("account_id") or {}).get("iban") or "?"
        currency = account.get("currency") or "?"
        tag = f"{psu_type}-{index}-{currency}"
        print(f"[{index}] {iban} {currency}  uid={uid}")

        for what in ("details", "balances"):
            status, payload = _get(f"/accounts/{uid}/{what}")
            if _rate_limited(status, payload):
                return 1
            if status == 200:
                _save(payload, f"{what}-{tag}")
            else:
                print(f"    {what} HTTP {status}: {json.dumps(payload)[:300]}")
                _save(payload, f"error-{what}-{tag}")

        transactions(uid, tag, months)
        print()

    print(f"raw responses in {config.RAW_DIR}")
    print("Next: python3 -m eb.load")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
