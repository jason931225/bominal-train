# PCI DSS + OWASP Security Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Codify and enforce PCI DSS + OWASP ASVS controls (L2 globally, L3 for payment/CDE/relay paths) with docs-first governance, critical code remediations, and blocking verification gates.

**Architecture:** First codify policy requirements in canonical docs, then enforce fail-closed controls at redaction/logging, crypto key-versioning, CVV Redis handling, provider egress, and safe metadata boundaries. Finally add verification gates in tests and CI.

**Tech Stack:** FastAPI, Python, Redis, ARQ, pytest, Docker Compose, markdown governance docs.

---

## Scope

1. Docs-first codification for CDE/PCI relay isolation and approval boundaries.
2. Critical code hardening for redaction, envelope decryption boundaries, provider egress, and persistence boundaries.
3. Compliance evidence and gate automation.

## Delivery checkpoints

1. Docs codification merged and validated with docs checks.
2. Security primitives hardened and covered by tests.
3. CI/runtime checks enforce fail-closed behavior.

## Acceptance criteria

- No raw PAN/CVV/provider payment payload crosses logs, queues, artifacts, or non-CDE persistence.
- `kek_version` is enforced at decrypt boundaries.
- Provider egress is allowlisted and SSRF resistant.
- CVV Redis TTL is bounded and persistence policy is enforced.
- Compliance matrix and scanner gates are present and passing.
