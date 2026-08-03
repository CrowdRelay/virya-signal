import base64
import importlib.util
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("configure-android-signing.py")
SPEC = importlib.util.spec_from_file_location("configure_android_signing", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class AndroidSigningTests(unittest.TestCase):
    def test_writes_private_keystore_and_escaped_properties(self):
        payload = b"virya-keystore" * 8
        environment = {
            "ANDROID_KEYSTORE_BASE64": base64.b64encode(payload).decode(),
            "ANDROID_KEY_ALIAS": "virya-upload",
            "ANDROID_KEY_PASSWORD": r"safe=password\value",
        }
        with tempfile.TemporaryDirectory() as directory, patch.dict(
            os.environ, environment, clear=True
        ):
            root = Path(directory)
            keystore = root / "key.jks"
            properties = root / "keystore.properties"
            MODULE.write_signing_files(keystore, properties)
            self.assertEqual(keystore.read_bytes(), payload)
            text = properties.read_text()
            self.assertIn(r"password=safe\=password\\value", text)
            self.assertEqual(keystore.stat().st_mode & 0o777, 0o600)
            self.assertEqual(properties.stat().st_mode & 0o777, 0o600)

    def test_rejects_malformed_base64(self):
        with self.assertRaisesRegex(ValueError, "valid base64"):
            MODULE.decode_keystore("not-base64!")

    def test_rejects_unsafe_alias(self):
        environment = {
            "ANDROID_KEYSTORE_BASE64": base64.b64encode(b"x" * 64).decode(),
            "ANDROID_KEY_ALIAS": "bad alias",
            "ANDROID_KEY_PASSWORD": "password",
        }
        with tempfile.TemporaryDirectory() as directory, patch.dict(
            os.environ, environment, clear=True
        ), self.assertRaisesRegex(ValueError, "unsupported"):
            root = Path(directory)
            MODULE.write_signing_files(root / "key.jks", root / "props")


if __name__ == "__main__":
    unittest.main()
