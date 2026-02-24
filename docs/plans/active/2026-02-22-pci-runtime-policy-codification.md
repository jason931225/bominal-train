# PCI Runtime Policy Codification Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Codify PCI-sensitive runtime isolation and related compliance controls into enforceable repository policy docs before code audit.

**Architecture:** This is a docs-first governance hardening pass. The policy baseline is centralized in `docs/humans/security/SECURITY.md` and then cross-enforced by `docs/agents/GUARDRAILS.md`, `docs/agents/PERMISSIONS.md`, and `docs/humans/engineering/ARCHITECTURE.md` to remove ambiguity and define deterministic PASS/FAIL gate criteria.

**Tech Stack:** Markdown docs, shell-based docs validation checks.

---

### Task 1: Codify PCI Relay Isolation and CDE Boundary

**Files:**
- Modify: `docs/humans/security/SECURITY.md`

**Steps:**
1. Add `Cardholder Data Environment (CDE) Boundary` section.
2. Add `PCI Relay Worker Isolation Policy` section with explicit MUST/MUST NOT constraints.
3. Define CRITICAL/BLOCK severity mapping for violations.
4. Verify wording uses fail-closed enforcement language.

**Verification:**
- Run: `rg -n "Cardholder Data Environment|PCI Relay Worker Isolation Policy|MUST NOT|CRITICAL|BLOCK" docs/humans/security/SECURITY.md`
- Expected: Section headings and enforcement terms present.

### Task 2: Codify Safe Metadata, Redis CVV Contract, and Logging Enforcement

**Files:**
- Modify: `docs/humans/security/SECURITY.md`

**Steps:**
1. Add `Safe Metadata Contract` allowlist/denylist policy.
2. Add `Redis CVV Storage and Persistence Contract` with TTL bounds and persistence/backup constraints.
3. Add `Logging Enforcement Architecture` with explicit boundary requirements.
4. Add `Provider Egress Allowlist Requirement` and SSRF baseline rule.

**Verification:**
- Run: `rg -n "Safe Metadata Contract|Redis CVV Storage and Persistence Contract|Logging Enforcement Architecture|Provider Egress Allowlist|SSRF" docs/humans/security/SECURITY.md`
- Expected: All required sections and controls present.

### Task 3: Align Guardrails, Permissions, and Architecture

**Files:**
- Modify: `docs/agents/GUARDRAILS.md`
- Modify: `docs/agents/PERMISSIONS.md`
- Modify: `docs/humans/engineering/ARCHITECTURE.md`

**Steps:**
1. Add hard-block PCI/runtime/payload leak rules to guardrails.
2. Add explicit permission-gate language for PCI-sensitive runtime operations.
3. Add architecture statements on CDE segmentation and payment-runtime boundaries.
4. Ensure language is consistent with `docs/humans/security/SECURITY.md`.

**Verification:**
- Run: `rg -n "PCI|CDE|egress|raw provider payload|CVV|card" docs/agents/GUARDRAILS.md docs/agents/PERMISSIONS.md docs/humans/engineering/ARCHITECTURE.md`
- Expected: Cross-doc policy consistency terms present.

### Task 4: Register Plan and Pointer Updates

**Files:**
- Modify: `docs/plans/active/README.md`
- Modify: `docs/README.md`

**Steps:**
1. Add the active plan file to `docs/plans/active/README.md`.
2. Add canonical pointer entry to `docs/README.md` for this plan.
3. Keep pointer format consistent with existing conventions.

**Verification:**
- Run: `bash infra/tests/test_docs_pointers.sh`
- Expected: Pass.

### Task 5: Update Changelog for Governance Changes

**Files:**
- Modify: `CHANGELOG.md`

**Steps:**
1. Add commit-based entry under `## Unreleased` describing policy codification scope.
2. Ensure entry aligns with Keep a Changelog category and SHA format.

**Verification:**
- Run: `bash infra/tests/test_changelog.sh`
- Expected: Pass.

### Task 6: Run Docs Governance Verification Suite

**Files:**
- No file changes

**Steps:**
1. Execute docs validation scripts.
2. Collect pass/fail results as evidence.

**Verification:**
- Run: `bash infra/tests/test_docs_pointers.sh`
- Run: `bash infra/tests/test_intent_routing.sh`
- Run: `bash infra/tests/test_docs_consistency.sh`
- Run: `bash infra/tests/test_changelog.sh`
- Expected: All pass.

### Task 7: Commit Batch by Logical Scope

**Files:**
- Staged files from Tasks 1-6

**Steps:**
1. Stage policy docs and governance updates.
2. Commit with scoped message.
3. Keep commits small and reviewable.

**Verification:**
- Run: `git show --name-only --oneline -1`
- Expected: Commit includes intended files only.

### Task 8: Final Review and Handoff

**Files:**
- No file changes

**Steps:**
1. Re-read modified docs for consistency.
2. Produce compliance-ready summary with exact changed sections.
3. Identify any follow-up enforcement work for code/CI.

**Verification:**
- Run: `git diff --stat HEAD~1..HEAD`
- Expected: Doc-only scope and coherent change set.
