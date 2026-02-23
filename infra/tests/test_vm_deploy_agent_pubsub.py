import json
import os
import stat
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
AGENT_PATH = REPO_ROOT / "infra" / "scripts" / "vm-deploy-agent-pubsub.sh"


def _write_exe(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


class TestVmDeployAgentPubSub(unittest.TestCase):
    def test_processes_one_message_extends_ack_and_acks_after_deploy(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            tdir = Path(td)
            bindir = tdir / "bin"
            bindir.mkdir(parents=True, exist_ok=True)

            calls_file = tdir / "gcloud_calls.txt"
            pulled_marker = tdir / "pulled_once"
            pull_response = tdir / "pull.json"
            deploy_args_file = tdir / "deploy_args.txt"
            deploy_env_file = tdir / "deploy_env.txt"

            pull_response.write_text(
                json.dumps(
                    [
                        {
                            "ackId": "ack-123",
                            "message": {
                                "attributes": {
                                    # Even if CI publishes commit-mode, the agent must deploy latest-only.
                                    "mode": "commit",
                                    "commit": "deadbeefcafebabe",
                                }
                            },
                        }
                    ]
                ),
                encoding="utf-8",
            )

            _write_exe(
                bindir / "gcloud",
                textwrap.dedent(
                    f"""\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    echo "gcloud $*" >> {calls_file!s}

                    if [[ "${{1:-}}" == "pubsub" && "${{2:-}}" == "subscriptions" && "${{3:-}}" == "pull" ]]; then
                      if [[ -f {pulled_marker!s} ]]; then
                        echo "[]"
                      else
                        cat {pull_response!s}
                        : > {pulled_marker!s}
                      fi
                      exit 0
                    fi

                    if [[ "${{1:-}}" == "pubsub" && "${{2:-}}" == "subscriptions" && ( "${{3:-}}" == "ack" || "${{3:-}}" == "modify-ack-deadline" ) ]]; then
                      exit 0
                    fi

                    echo "unexpected gcloud invocation: $*" >&2
                    exit 1
                    """
                ),
            )

            _write_exe(
                bindir / "git",
                "#!/usr/bin/env bash\nset -euo pipefail\nexit 0\n",
            )

            _write_exe(
                bindir / "flock",
                "#!/usr/bin/env bash\nset -euo pipefail\n# Test stub: no-op lock.\nexit 0\n",
            )

            deploy_script = tdir / "deploy.sh"
            _write_exe(
                deploy_script,
                textwrap.dedent(
                    f"""\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    printf '%s' "$*" > {deploy_args_file!s}
                    {{
                      printf 'API_IMAGE=%s\\n' "${{API_IMAGE:-}}"
                      printf 'WORKER_IMAGE=%s\\n' "${{WORKER_IMAGE:-}}"
                      printf 'WEB_IMAGE=%s\\n' "${{WEB_IMAGE:-}}"
                    }} > {deploy_env_file!s}
                    exit 0
                    """
                ),
            )

            env = os.environ.copy()
            env["PATH"] = f"{bindir}{os.pathsep}{env.get('PATH','')}"
            env["GCP_PROJECT_ID"] = "bominal"
            env["DEPLOY_SUBSCRIPTION"] = "sub1"
            env["REPO_DIR"] = str(tdir / "repo")
            env["DEPLOY_SCRIPT"] = str(deploy_script)
            env["ALLOW_NONCANONICAL_DEPLOY_SCRIPT"] = "true"
            env["LOCK_FILE"] = str(tdir / "lockfile")
            env["SLEEP_SECONDS"] = "0"
            env["DEPLOY_AGENT_ONCE"] = "1"
            env["ACK_DEADLINE_SECONDS"] = "600"
            env["ACK_EXTEND_INTERVAL_SECONDS"] = "60"

            Path(env["REPO_DIR"]).mkdir(parents=True, exist_ok=True)

            try:
                result = subprocess.run(
                    ["bash", str(AGENT_PATH)],
                    env=env,
                    text=True,
                    capture_output=True,
                    timeout=10,
                    check=False,
                )
            except subprocess.TimeoutExpired as exc:
                self.fail(f"agent did not exit in once mode: {exc}")

            self.assertEqual(result.returncode, 0, msg=f"stderr={result.stderr!r}\nstdout={result.stdout!r}")

            # Latest-only deploy: no args should be passed even if message mode=commit.
            self.assertTrue(deploy_args_file.exists(), msg="deploy script was not invoked")
            self.assertEqual(deploy_args_file.read_text(encoding="utf-8").strip(), "")
            env_dump = deploy_env_file.read_text(encoding="utf-8")
            self.assertIn("API_IMAGE=", env_dump)
            self.assertIn("WORKER_IMAGE=", env_dump)
            self.assertIn("WEB_IMAGE=", env_dump)
            self.assertIn("API_IMAGE=\n", env_dump)
            self.assertIn("WORKER_IMAGE=\n", env_dump)
            self.assertIn("WEB_IMAGE=\n", env_dump)

            calls = calls_file.read_text(encoding="utf-8")
            self.assertIn("pubsub subscriptions pull sub1", calls)
            self.assertIn("pubsub subscriptions modify-ack-deadline sub1", calls)
            self.assertIn("pubsub subscriptions ack sub1", calls)

    def test_applies_per_service_image_overrides(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            tdir = Path(td)
            bindir = tdir / "bin"
            bindir.mkdir(parents=True, exist_ok=True)

            calls_file = tdir / "gcloud_calls.txt"
            pulled_marker = tdir / "pulled_once"
            pull_response = tdir / "pull.json"
            deploy_env_file = tdir / "deploy_env.txt"

            pull_response.write_text(
                json.dumps(
                    [
                        {
                            "ackId": "ack-123",
                            "message": {
                                "attributes": {
                                    "mode": "latest",
                                    "api_image": "ghcr.io/example/bominal/api:abc",
                                    "worker_image": "ghcr.io/example/bominal/api:abc",
                                    "web_image": "ghcr.io/example/bominal/web:abc",
                                }
                            },
                        }
                    ]
                ),
                encoding="utf-8",
            )

            _write_exe(
                bindir / "gcloud",
                textwrap.dedent(
                    f"""\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    echo "gcloud $*" >> {calls_file!s}

                    if [[ "${{1:-}}" == "pubsub" && "${{2:-}}" == "subscriptions" && "${{3:-}}" == "pull" ]]; then
                      if [[ -f {pulled_marker!s} ]]; then
                        echo "[]"
                      else
                        cat {pull_response!s}
                        : > {pulled_marker!s}
                      fi
                      exit 0
                    fi

                    if [[ "${{1:-}}" == "pubsub" && "${{2:-}}" == "subscriptions" && ( "${{3:-}}" == "ack" || "${{3:-}}" == "modify-ack-deadline" ) ]]; then
                      exit 0
                    fi

                    echo "unexpected gcloud invocation: $*" >&2
                    exit 1
                    """
                ),
            )
            _write_exe(
                bindir / "git",
                "#!/usr/bin/env bash\nset -euo pipefail\nexit 0\n",
            )
            _write_exe(
                bindir / "flock",
                "#!/usr/bin/env bash\nset -euo pipefail\nexit 0\n",
            )

            deploy_script = tdir / "deploy.sh"
            _write_exe(
                deploy_script,
                textwrap.dedent(
                    f"""\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    {{
                      printf 'API_IMAGE=%s\\n' "${{API_IMAGE:-}}"
                      printf 'WORKER_IMAGE=%s\\n' "${{WORKER_IMAGE:-}}"
                      printf 'WEB_IMAGE=%s\\n' "${{WEB_IMAGE:-}}"
                    }} > {deploy_env_file!s}
                    exit 0
                    """
                ),
            )

            env = os.environ.copy()
            env["PATH"] = f"{bindir}{os.pathsep}{env.get('PATH','')}"
            env["GCP_PROJECT_ID"] = "bominal"
            env["DEPLOY_SUBSCRIPTION"] = "sub1"
            env["REPO_DIR"] = str(tdir / "repo")
            env["DEPLOY_SCRIPT"] = str(deploy_script)
            env["ALLOW_NONCANONICAL_DEPLOY_SCRIPT"] = "true"
            env["LOCK_FILE"] = str(tdir / "lockfile")
            env["SLEEP_SECONDS"] = "0"
            env["DEPLOY_AGENT_ONCE"] = "1"

            Path(env["REPO_DIR"]).mkdir(parents=True, exist_ok=True)

            result = subprocess.run(
                ["bash", str(AGENT_PATH)],
                env=env,
                text=True,
                capture_output=True,
                timeout=10,
                check=False,
            )
            self.assertEqual(result.returncode, 0, msg=f"stderr={result.stderr!r}\nstdout={result.stdout!r}")

            env_dump = deploy_env_file.read_text(encoding="utf-8")
            self.assertIn("API_IMAGE=ghcr.io/example/bominal/api:abc", env_dump)
            self.assertIn("WORKER_IMAGE=ghcr.io/example/bominal/api:abc", env_dump)
            self.assertIn("WEB_IMAGE=ghcr.io/example/bominal/web:abc", env_dump)
