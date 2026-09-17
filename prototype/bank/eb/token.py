"""Mint the RS256 JWT that authorizes every Enable Banking call.

The token is signed locally against the app's RSA key. There is no token
endpoint and no round trip, so a bad key shows up as a 401 on the first real
call, not as a login failure.

Run directly to inspect what is being signed:
    python3 -m eb.token
"""

import datetime as dt
import json
import time

import jwt as pyjwt

from . import config

ISSUER = "enablebanking.com"
AUDIENCE = "api.enablebanking.com"

# The documented maximum is 86400s. An hour is plenty and bounds the damage
# if a token leaks into a log somewhere.
TTL_SECONDS = 3600
# Backdated so a few seconds of clock skew against their servers is harmless.
SKEW_SECONDS = 30


def mint() -> str:
    now = int(time.time())
    return pyjwt.encode(
        {
            "iss": ISSUER,
            "aud": AUDIENCE,
            "iat": now - SKEW_SECONDS,
            "exp": now + TTL_SECONDS,
        },
        config.private_key(),
        algorithm="RS256",
        headers={"kid": config.application_id()},
    )


def auth_header() -> dict[str, str]:
    return {"Authorization": f"Bearer {mint()}"}


def _describe(token: str) -> None:
    """Print the token's shape without trusting it — decode, don't verify."""
    header = pyjwt.get_unverified_header(token)
    claims = pyjwt.decode(token, options={"verify_signature": False}, audience=AUDIENCE)
    print("header:", json.dumps(header, indent=2))
    print("claims:", json.dumps(claims, indent=2))
    exp = dt.datetime.fromtimestamp(claims["exp"], dt.UTC)
    print(f"\nsegments: {len(token.split('.'))}  (must be 3)")
    print(f"expires:  {exp.isoformat()}  (in {claims['exp'] - int(time.time())}s)")
    print(f"length:   {len(token)} chars")


if __name__ == "__main__":
    _describe(mint())
    print("\nSigning works. Nothing has been sent anywhere.")
