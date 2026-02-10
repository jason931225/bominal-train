# Pub/Sub CI Deploy Agent Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace GitHub Actions SSH-based production deploy with a Pub/Sub publish (CI) + pull-based deploy agent on the VM (no inbound SSH), while continuing to run `infra/scripts/deploy-zero-downtime.sh` on the VM.

**Architecture:** GitHub Actions (authenticated to GCP via WIF) publishes a deploy request message to a Pub/Sub topic. A small systemd service on the `bominal-deploy` VM pulls a subscription, checks out the requested commit for infra config alignment, runs the existing zero-downtime deploy script, and ACKs the message only after a successful deploy.

**Tech Stack:** GitHub Actions, `google-github-actions/auth`, `setup-gcloud`, Pub/Sub, `gcloud` CLI, systemd, bash, git.

---

### Task 1: Add Pub/Sub Deploy Trigger to GitHub Actions

**Files:**
- Modify: `.github/workflows/deploy.yml`

**Step 1: Update workflow to publish to Pub/Sub (no SSH)**
- Replace SSH key generation + `ssh-compute@v1` usage with:
  - compute `mode` and `commit`
  - `gcloud pubsub topics publish ...` with a JSON message and attributes

**Step 2: Sanity-check YAML**
- Run: `python3 -c 'import yaml,sys; yaml.safe_load(open(\".github/workflows/deploy.yml\")); print(\"ok\")'`
- Expected: prints `ok`

**Step 3: Commit**
- Run: `git add .github/workflows/deploy.yml`
- Run: `git commit -m "ci: publish deploy requests to Pub/Sub"`

---

### Task 2: Add VM Pub/Sub Pull Deploy Agent Script

**Files:**
- Create: `infra/scripts/vm-deploy-agent-pubsub.sh`

**Step 1: Implement the agent loop**
- Requirements:
  - acquire a lock (`flock`) to prevent concurrent deploys
  - pull exactly one message (no auto-ack)
  - extract `ackId` + attributes (`mode`, `commit`)
  - `git fetch` + `git checkout` the commit (or `origin/main` for latest)
  - run `infra/scripts/deploy-zero-downtime.sh` (latest) or with commit tag (commit mode)
  - ACK only on success; do not ACK on failure

**Step 2: Validate shell**
- Run: `bash -n infra/scripts/vm-deploy-agent-pubsub.sh`
- Expected: exit code `0`

**Step 3: Commit**
- Run: `git add infra/scripts/vm-deploy-agent-pubsub.sh`
- Run: `git commit -m "infra: add Pub/Sub pull-based deploy agent script"`

---

### Task 3: Add systemd Unit Template + Docs

**Files:**
- Create: `infra/systemd/bominal-deploy-agent.service`
- Modify: `docs/DEPLOYMENT.md`

**Step 1: Create systemd unit**
- Use `EnvironmentFile=/etc/bominal/deploy-agent.env`
- Run as `bominal` user
- Start after networking + docker
- Restart on failure

**Step 2: Update deployment docs**
- Document:
  - one-time Pub/Sub topic + subscription creation
  - IAM bindings (publisher for GitHub Actions SA; subscriber for VM SA)
  - VM install steps for the systemd unit + env file
  - operational commands (`journalctl -u ...`, disable/stop)
  - manual fallback deploy command

**Step 3: Commit**
- Run: `git add infra/systemd/bominal-deploy-agent.service docs/DEPLOYMENT.md`
- Run: `git commit -m "docs/infra: document Pub/Sub deploy agent and systemd unit"`

---

### Task 4: Local Verification (Repo-Only)

**Step 1: Validate workflow YAML**
- Run: `python3 -c 'import yaml; yaml.safe_load(open(\".github/workflows/deploy.yml\")); print(\"ok\")'`
- Expected: prints `ok`

**Step 2: Validate scripts**
- Run: `bash -n infra/scripts/vm-deploy-agent-pubsub.sh`
- Expected: exit code `0`

---

### Task 5: Production Verification (On VM)

**Step 1: Install and start agent**
- Apply the steps documented in `docs/DEPLOYMENT.md`

**Step 2: Publish a test message**
- Use `gcloud pubsub topics publish ...` and confirm the agent runs a deploy once.

**Step 3: Confirm health**
- Run: `curl https://www.bominal.com/health`
- Expected: HTTP 200 (and normal app payload)

