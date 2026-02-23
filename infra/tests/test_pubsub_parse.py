import json
import subprocess
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT_PATH = REPO_ROOT / "infra" / "scripts" / "pubsub_parse.py"


def run_parser(payload: str) -> str:
    result = subprocess.run(
        [sys.executable, str(SCRIPT_PATH)],
        input=payload,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise AssertionError(f"parser failed: rc={result.returncode} stderr={result.stderr!r}")
    return result.stdout


class TestPubSubParse(unittest.TestCase):
    def test_parses_ack_and_attributes(self) -> None:
        payload = json.dumps(
            [
                {
                    "ackId": "ack-1",
                    "message": {
                        "attributes": {
                            "mode": "latest",
                            "commit": "abc 123",
                            "api_gateway_image": "ghcr.io/example/api-gateway:abc123",
                        }
                    },
                }
            ]
        )

        output = run_parser(payload)
        self.assertEqual(
            output.splitlines(),
            [
                "ACK_ID=ack-1",
                "DEPLOY_MODE=latest",
                "DEPLOY_COMMIT='abc 123'",
                "DEPLOY_API_GATEWAY_IMAGE=ghcr.io/example/api-gateway:abc123",
                "DEPLOY_API_TRAIN_IMAGE=''",
                "DEPLOY_API_RESTAURANT_IMAGE=''",
                "DEPLOY_WORKER_TRAIN_IMAGE=''",
                "DEPLOY_WORKER_RESTAURANT_IMAGE=''",
                "DEPLOY_WEB_IMAGE=''",
            ],
        )

    def test_defaults_when_attributes_missing(self) -> None:
        payload = json.dumps(
            [
                {
                    "ackId": "ack-2",
                    "message": {},
                }
            ]
        )

        output = run_parser(payload)
        self.assertEqual(
            output.splitlines(),
            [
                "ACK_ID=ack-2",
                "DEPLOY_MODE=latest",
                "DEPLOY_COMMIT=''",
                "DEPLOY_API_GATEWAY_IMAGE=''",
                "DEPLOY_API_TRAIN_IMAGE=''",
                "DEPLOY_API_RESTAURANT_IMAGE=''",
                "DEPLOY_WORKER_TRAIN_IMAGE=''",
                "DEPLOY_WORKER_RESTAURANT_IMAGE=''",
                "DEPLOY_WEB_IMAGE=''",
            ],
        )

    def test_defaults_when_no_messages(self) -> None:
        output = run_parser("[]")
        self.assertEqual(
            output.splitlines(),
            [
                "ACK_ID=''",
                "DEPLOY_MODE=latest",
                "DEPLOY_COMMIT=''",
                "DEPLOY_API_GATEWAY_IMAGE=''",
                "DEPLOY_API_TRAIN_IMAGE=''",
                "DEPLOY_API_RESTAURANT_IMAGE=''",
                "DEPLOY_WORKER_TRAIN_IMAGE=''",
                "DEPLOY_WORKER_RESTAURANT_IMAGE=''",
                "DEPLOY_WEB_IMAGE=''",
            ],
        )
