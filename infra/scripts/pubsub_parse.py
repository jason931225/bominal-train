#!/usr/bin/env python3
import json
import shlex
import sys
from typing import Any, Mapping


def _shell_assign(key: str, value: str) -> str:
    return f"{key}={shlex.quote(value)}"


def _first_message(payload: Any) -> Mapping[str, Any]:
    if isinstance(payload, list):
        if payload:
            msg = payload[0]
            if isinstance(msg, dict):
                return msg
    return {}


def main() -> int:
    raw = sys.stdin.read()
    if not raw.strip():
        payload: Any = []
    else:
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError as exc:
            print(f"invalid json input: {exc}", file=sys.stderr)
            return 2

    msg = _first_message(payload)
    ack_id = str(msg.get("ackId", ""))

    attrs: Mapping[str, Any] = {}
    body = msg.get("message")
    if isinstance(body, dict):
        maybe_attrs = body.get("attributes")
        if isinstance(maybe_attrs, dict):
            attrs = maybe_attrs

    deploy_mode = str(attrs.get("mode", "latest"))
    deploy_commit = str(attrs.get("commit", ""))
    deploy_api_image = str(attrs.get("api_image", ""))
    deploy_worker_image = str(attrs.get("worker_image", ""))
    # Backward compatibility with split-runtime deploy payloads.
    if not deploy_api_image:
        deploy_api_image = str(
            attrs.get("api_gateway_image", "")
            or attrs.get("api_train_image", "")
            or attrs.get("api_restaurant_image", "")
        )
    if not deploy_worker_image:
        deploy_worker_image = str(
            attrs.get("worker_image", "")
            or attrs.get("worker_train_image", "")
            or attrs.get("worker_restaurant_image", "")
            or deploy_api_image
        )
    deploy_web_image = str(attrs.get("web_image", ""))

    lines = [
        _shell_assign("ACK_ID", ack_id),
        _shell_assign("DEPLOY_MODE", deploy_mode),
        _shell_assign("DEPLOY_COMMIT", deploy_commit),
        _shell_assign("DEPLOY_API_IMAGE", deploy_api_image),
        _shell_assign("DEPLOY_WORKER_IMAGE", deploy_worker_image),
        _shell_assign("DEPLOY_WEB_IMAGE", deploy_web_image),
        # Deprecated compatibility exports (kept for staged migration safety).
        _shell_assign("DEPLOY_API_GATEWAY_IMAGE", deploy_api_image),
        _shell_assign("DEPLOY_API_TRAIN_IMAGE", ""),
        _shell_assign("DEPLOY_API_RESTAURANT_IMAGE", ""),
        _shell_assign("DEPLOY_WORKER_TRAIN_IMAGE", deploy_worker_image),
        _shell_assign("DEPLOY_WORKER_RESTAURANT_IMAGE", ""),
    ]
    sys.stdout.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
