#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import binascii
import os
import re
from pathlib import Path


ALIAS_PATTERN = re.compile(r"[A-Za-z0-9_.-]{1,128}\Z")


def required_env(name: str) -> str:
    value = os.environ.get(name, "")
    if not value:
        raise ValueError(f"missing {name}")
    return value


def decode_keystore(value: str) -> bytes:
    compact = "".join(value.split())
    try:
        payload = base64.b64decode(compact, validate=True)
    except (binascii.Error, ValueError) as error:
        raise ValueError("ANDROID_KEYSTORE_BASE64 is not valid base64") from error
    if len(payload) < 64:
        raise ValueError("decoded Android keystore is unexpectedly small")
    return payload


def properties_escape(value: str) -> str:
    if any(character in value for character in "\r\n\0"):
        raise ValueError("Android signing values cannot contain control characters")
    return value.replace("\\", "\\\\").replace(":", "\\:").replace("=", "\\=")


def write_signing_files(keystore: Path, properties: Path) -> None:
    alias = required_env("ANDROID_KEY_ALIAS")
    password = required_env("ANDROID_KEY_PASSWORD")
    if not ALIAS_PATTERN.fullmatch(alias):
        raise ValueError("ANDROID_KEY_ALIAS contains unsupported characters")
    payload = decode_keystore(required_env("ANDROID_KEYSTORE_BASE64"))
    keystore.parent.mkdir(parents=True, exist_ok=True)
    properties.parent.mkdir(parents=True, exist_ok=True)
    keystore.write_bytes(payload)
    properties.write_text(
        "\n".join(
            [
                f"keyAlias={properties_escape(alias)}",
                f"password={properties_escape(password)}",
                f"storeFile={properties_escape(str(keystore.resolve()))}",
                "",
            ]
        ),
        encoding="utf-8",
    )
    os.chmod(keystore, 0o600)
    os.chmod(properties, 0o600)


def main() -> int:
    parser = argparse.ArgumentParser(description="Create private Android signing inputs")
    parser.add_argument("keystore", type=Path)
    parser.add_argument("properties", type=Path)
    args = parser.parse_args()
    try:
        write_signing_files(args.keystore, args.properties)
    except ValueError as error:
        parser.error(str(error))
    print("Android signing files prepared")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
