# 2026-03-04 Refactor Baseline Evidence (Template)

Purpose: Capture pre-refactor repository hardening evidence for `bominal`.

## Metadata
- Document owner: `<name>`
- Prepared at (UTC): `<YYYY-MM-DDTHH:MM:SSZ>`
- Repository: `<owner/repo>`
- Baseline branch: `<branch>`
- Baseline commit SHA: `<sha>`

## Evidence Rules
- Do not include secrets, tokens, passwords, card data, or sensitive payloads.
- Use UTC timestamps for every command/evidence item.
- Paste output verbatim except explicitly redacted sensitive substrings marked `[REDACTED]`.

## 1) Branch Protection
- Verified at (UTC): `<YYYY-MM-DDTHH:MM:SSZ>`
- Target branch: `<branch>`

### Commands
```bash
# Example (adjust owner/repo/branch):
gh api repos/<owner>/<repo>/branches/<branch>/protection
```

### Output
```text
<PASTE_BRANCH_PROTECTION_OUTPUT>
```

### Baseline Notes
- Protected branch enabled: `<yes/no>`
- Force push blocked: `<yes/no>`
- Deletion blocked: `<yes/no>`
- Admin enforcement: `<yes/no>`
- Notes: `<free text>`

## 2) Required Checks
- Verified at (UTC): `<YYYY-MM-DDTHH:MM:SSZ>`
- Target branch: `<branch>`

### Commands
```bash
# Example:
gh api repos/<owner>/<repo>/branches/<branch>/protection/required_status_checks
```

### Output
```text
<PASTE_REQUIRED_CHECKS_OUTPUT>
```

### Baseline Notes
- Strict status checks required: `<yes/no>`
- Required checks list:
  - `<check-1>`
  - `<check-2>`
  - `<check-...>`
- Notes: `<free text>`

## 3) PR Review Policy
- Verified at (UTC): `<YYYY-MM-DDTHH:MM:SSZ>`
- Target branch: `<branch>`

### Commands
```bash
# Example:
gh api repos/<owner>/<repo>/branches/<branch>/protection/required_pull_request_reviews
```

### Output
```text
<PASTE_PR_REVIEW_POLICY_OUTPUT>
```

### Baseline Notes
- PR required before merge: `<yes/no>`
- Required approving reviews: `<number>`
- Dismiss stale approvals on new commit: `<yes/no>`
- Code owner review required: `<yes/no>`
- Last-push approval required: `<yes/no>`
- Notes: `<free text>`

## 4) Deployment Approval Gate
- Verified at (UTC): `<YYYY-MM-DDTHH:MM:SSZ>`
- Deployment environment: `<env-name>`

### Commands
```bash
# Example:
gh api repos/<owner>/<repo>/environments/<env-name>
```

### Output
```text
<PASTE_DEPLOYMENT_GATE_OUTPUT>
```

### Baseline Notes
- Approval gate enabled: `<yes/no>`
- Required reviewers:
  - `<reviewer/team-1>`
  - `<reviewer/team-...>`
- Self-approval blocked: `<yes/no>`
- Wait timer configured: `<yes/no>`
- Notes: `<free text>`

## Sign-Off
- Evidence captured by: `<name>`
- Captured at (UTC): `<YYYY-MM-DDTHH:MM:SSZ>`
- Reviewer: `<name>`
- Review timestamp (UTC): `<YYYY-MM-DDTHH:MM:SSZ>`
