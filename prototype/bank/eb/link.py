"""Consent flow: authorize one Erste account and keep the session.

    python3 -m eb.link business
    python3 -m eb.link personal

Prints the bank's authorization URL. Open it, authenticate, and the bank sends you
to https://finance.proximity.is/connect/callback, which shows the code. Paste it
back here.

The code is single-use and short-lived. If it expires, start over — nothing is
consumed by a failed attempt except the code itself.
"""

import datetime as dt
import json
import secrets
import sys

import requests

from . import config, token

TIMEOUT = 30
ASPSP = {"name": "Erste & Steiermärkische Bank", "country": "HR"}
REDIRECT_URL = "https://finance.proximity.is/connect/callback"
# Erste's maximum_consent_validity is 15552000s (180d). Stay a day inside it so a
# slow authorization cannot land past the limit and be refused.
CONSENT_DAYS = 179

SESSION_DIR = config.RAW_DIR.parent / "sessions"


def start(psu_type: str) -> tuple[str, str]:
    state = secrets.token_urlsafe(24)
    valid_until = dt.datetime.now(dt.UTC) + dt.timedelta(days=CONSENT_DAYS)
    body = {
        "access": {"valid_until": valid_until.isoformat().replace("+00:00", "Z")},
        "aspsp": ASPSP,
        "state": state,
        "redirect_url": REDIRECT_URL,
        "psu_type": psu_type,
        "language": "hr",
    }
    response = requests.post(
        f"{config.BASE_URL}/auth", json=body, headers=token.auth_header(), timeout=TIMEOUT
    )
    if response.status_code not in (200, 201):
        config.die(f"POST /auth -> HTTP {response.status_code}\n{response.text[:1500]}")
    payload = response.json()
    _save(payload, f"auth-{psu_type}")
    return payload["url"], state


def exchange(code: str, psu_type: str) -> dict:
    response = requests.post(
        f"{config.BASE_URL}/sessions",
        json={"code": code},
        headers=token.auth_header(),
        timeout=TIMEOUT,
    )
    if response.status_code not in (200, 201):
        config.die(
            f"POST /sessions -> HTTP {response.status_code}\n{response.text[:1500]}\n\n"
            "  The code is single-use and expires within minutes. Run the flow again."
        )
    payload = response.json()
    _save(payload, f"session-{psu_type}")

    SESSION_DIR.mkdir(parents=True, exist_ok=True)
    path = SESSION_DIR / f"{psu_type}.json"
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False))
    print(f"\nsession saved to {path}")
    return payload


def _save(payload: dict, name: str) -> None:
    config.RAW_DIR.mkdir(parents=True, exist_ok=True)
    (config.RAW_DIR / f"{name}.json").write_text(json.dumps(payload, indent=2, ensure_ascii=False))


def _describe(session: dict) -> None:
    print(f"\nsession_id : {session.get('session_id')}")
    print(f"psu_type   : {session.get('psu_type')}")
    print(f"valid until: {(session.get('access') or {}).get('valid_until')}")
    accounts = session.get("accounts") or []
    print(f"\n{len(accounts)} account(s):")
    for account in accounts:
        ident = account.get("account_id") or {}
        print(
            f"  {ident.get('iban') or ident.get('other') or '?'}  "
            f"{account.get('currency', '?')}  "
            f"{account.get('cash_account_type', '?')}  "
            f"{account.get('name') or ''}"
        )
        print(f"    uid: {account.get('uid')}")


def main() -> int:
    """Two steps, neither interactive.

        python3 -m eb.link business          # prints the bank URL
        python3 -m eb.link business <code>   # exchanges the code from the callback
    """
    psu_type = sys.argv[1] if len(sys.argv) > 1 else "business"
    if psu_type not in ("business", "personal"):
        config.die("usage: python3 -m eb.link [business|personal] [code]")

    if len(sys.argv) > 2:
        _describe(exchange(sys.argv[2].strip(), psu_type))
        print(f"\nNext: python3 -m eb.pull {psu_type}")
        return 0

    url, state = start(psu_type)
    print(f"psu_type: {psu_type}")
    print("\nOpen this in a browser and authenticate with Erste:\n")
    print(f"  {url}\n")
    print(f"Expected state on return: {state}")
    print("The callback page shows the code. Then run:\n")
    print(f"  python3 -m eb.link {psu_type} <code>")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
