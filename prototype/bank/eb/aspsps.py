"""THE GATE.

One authenticated call: which banks does Enable Banking reach in Croatia, and
does Erste expose *business* accounts? If it doesn't, the whole Enable Banking
route is dead and nothing else in the plan should be built against it.

    python3 -m eb.aspsps

Exit codes: 0 Erste supports business, 1 it doesn't, 2 the call failed.
"""

import json
import sys

import requests

from . import config, token

TIMEOUT = 30


def fetch(country: str) -> dict:
    url = f"{config.BASE_URL}/aspsps"
    response = requests.get(
        url,
        params={"country": country},
        headers=token.auth_header(),
        timeout=TIMEOUT,
    )
    if response.status_code != 200:
        body = response.text[:2000]
        config.die(
            f"GET {url}?country={country} -> HTTP {response.status_code}\n"
            f"{body}\n\n"
            "  401/403: the application id or the key is wrong, or the app is not activated.\n"
            "  404:     wrong base URL."
        )
    return response.json()


def save(payload: dict, country: str) -> str:
    config.RAW_DIR.mkdir(parents=True, exist_ok=True)
    path = config.RAW_DIR / f"aspsps-{country}.json"
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False))
    return str(path)


def _psu_types(aspsp: dict) -> list[str]:
    # Documented as psu_types; tolerate the singular in case the shape differs.
    types = aspsp.get("psu_types") or aspsp.get("psu_type") or []
    return [types] if isinstance(types, str) else list(types)


def report(payload: dict, country: str) -> int:
    banks = payload.get("aspsps", payload if isinstance(payload, list) else [])
    if not banks:
        print("No ASPSPs returned. Raw payload keys:", list(payload)[:10])
        return 2

    print(f"{len(banks)} ASPSPs in {country}\n")
    header = f"{'name':<34} {'psu_types':<22} {'consent':>9}  beta"
    print(header)
    print("-" * len(header))
    for bank in sorted(banks, key=lambda b: str(b.get("name", ""))):
        validity = bank.get("maximum_consent_validity")
        days = f"{int(validity) // 86400}d" if validity else "-"
        print(
            f"{str(bank.get('name', '?')):<34} "
            f"{','.join(_psu_types(bank)) or '-':<22} "
            f"{days:>9}  {'yes' if bank.get('beta') else ''}"
        )

    erste = [b for b in banks if "erste" in str(b.get("name", "")).lower()]
    print()
    if not erste:
        print("VERDICT: Erste does not appear in this list at all.")
        print("         Fall back to Salt Edge, which lists Erste&Steiermarkische HR.")
        return 1

    verdict = 1
    for bank in erste:
        print(f"=== {bank.get('name')} ===")
        print(json.dumps(bank, indent=2, ensure_ascii=False))
        types = [t.lower() for t in _psu_types(bank)]
        print()
        print(f"  business accounts : {'YES' if 'business' in types else 'NO'}")
        print(f"  personal accounts : {'YES' if 'personal' in types else 'NO'}")
        headers = bank.get("required_psu_headers") or []
        if headers:
            # Psu-Ip-Address here means data can only be fetched while the user
            # is actually online, which would break unattended scheduled syncs.
            print(f"  required headers  : {', '.join(headers)}")
        if "business" in types:
            verdict = 0

    print()
    print(
        "VERDICT: Erste exposes business accounts. Proceed to the consent flow."
        if verdict == 0
        else "VERDICT: Erste is reachable but NOT for business accounts. Stop here."
    )
    return verdict


def main() -> int:
    country = sys.argv[1] if len(sys.argv) > 1 else config.COUNTRY
    payload = fetch(country)
    path = save(payload, country)
    code = report(payload, country)
    print(f"\nraw response saved to {path}")
    return code


if __name__ == "__main__":
    raise SystemExit(main())
