"""Where the credentials live. Nothing here is secret; the key stays on disk."""

import os
import pathlib
import sys

BASE_URL = os.environ.get("EB_BASE_URL", "https://api.enablebanking.com")
COUNTRY = os.environ.get("EB_COUNTRY", "HR")

DEFAULT_KEY = pathlib.Path.home() / ".config" / "enablebanking" / "private.key"
KEY_PATH = pathlib.Path(os.environ.get("EB_PRIVATE_KEY", DEFAULT_KEY))

RAW_DIR = pathlib.Path(__file__).resolve().parent.parent / "data" / "raw"


def application_id() -> str:
    """The JWT `kid`. Half the credential, so it comes from the environment."""
    app_id = os.environ.get("EB_APPLICATION_ID", "").strip()
    if not app_id:
        die(
            "EB_APPLICATION_ID is not set.\n"
            "  Get it from the Enable Banking control panel after registering the app, then:\n"
            "    export EB_APPLICATION_ID=<the-uuid>"
        )
    return app_id


def private_key() -> bytes:
    if not KEY_PATH.exists():
        die(
            f"No private key at {KEY_PATH}\n"
            "  Set EB_PRIVATE_KEY to its path, or generate one with:\n"
            "    openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:4096 -out private.key"
        )
    data = KEY_PATH.read_bytes()
    if b"BEGIN RSA PRIVATE KEY" in data:
        # Works here, but the Rust implementation uses ring, which takes PKCS#8 only.
        # Flag it now rather than discovering it at the port.
        warn(
            f"{KEY_PATH} is PKCS#1. Python accepts it; ring (the Rust path) will not.\n"
            "  Convert with: openssl pkcs8 -topk8 -nocrypt -in private.key -out private.pk8.pem"
        )
    return data


def die(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(2)


def warn(message: str) -> None:
    print(f"warning: {message}", file=sys.stderr)
