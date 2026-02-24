> Archive note (2026-02-14): Completed and moved from `docs/plans/active/` during Stage 8 closure.
> See `docs/plans/archive/2026-02-14-program-closure-report.md` for final status.

# Bominal Grand Restructure Plan (Umbrella)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Track the full restructure program and delegate implementation to stage-level executable plans.

**Architecture:** This is an umbrella tracker. Execution happens only through the stage plans listed below. Lock/request workflow, docs-first/docs-last gates, and changelog policy are enforced by `docs/agents/EXECUTION_PROTOCOL.md`.

**Tech Stack:** FastAPI, SQLAlchemy, Redis/arq, Docker Compose, Bash, pytest.

---

## Execution Contract

1. This file is non-executable by itself.
2. Historical note: execution was performed one stage plan at a time from `docs/plans/active/` (now archived).
3. Every stage execution must follow:
- `docs/agents/EXECUTION_PROTOCOL.md`
- `docs/governance/CHANGE_MANAGEMENT.md`
- `docs/governance/CHANGE_MANAGEMENT.md`

## Stage Plan Index

### Stage 1: Backend hardening baseline

Status: Completed.
Reference: `docs/todo/backend-production-readiness.md`

### Stage 2: Worker split + queue contracts

Status: Completed.
Execution plan: `docs/plans/archive/2026-02-14-stage2-worker-split-queue-contracts.md`

### Stage 3: Restaurant partial exposure

Status: Completed.
Execution plan: `docs/plans/archive/2026-02-14-stage3-restaurant-partial-exposure.md`

### Stage 4: Restaurant policy enforcement

Status: Completed.
Execution plan: `docs/plans/archive/2026-02-14-stage4-restaurant-policy-enforcement.md`

### Stage 5: Infra deploy hardening

Status: Completed.
Execution plan: `docs/plans/archive/2026-02-14-stage5-infra-deploy-hardening.md`

### Stage 6: Safe deprecation cleanup

Status: Completed.
Execution plan: `docs/plans/archive/2026-02-14-stage6-safe-deprecation-cleanup.md`

### Stage 7: Docs canonization

Status: Completed.
Execution plan: `docs/plans/archive/2026-02-14-stage7-docs-canonization.md`

## Program-Level Verification Gates

Per stage completion:
- stage-specific tests pass
- relevant backend/web/infra checks pass
- docs validators pass
- lock/request ledgers are consistent

Program completion:
- all stage plan statuses marked complete
- no unresolved `OPEN` requests in `docs/governance/CHANGE_MANAGEMENT.md`
- no lingering `ACTIVE` locks in `docs/governance/CHANGE_MANAGEMENT.md`
- backlog status report updated: `docs/plans/archive/2026-02-14-backlog-status-report.md`
