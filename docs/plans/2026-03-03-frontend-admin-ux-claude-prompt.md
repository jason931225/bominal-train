# 2026-03-03 Frontend + Admin UX Claude Prompt Package

## Purpose

Provide a ready-to-use Claude prompt to design a mobile-first + desktop-friendly
frontend for `bominal`, including a separate admin app with comprehensive
maintenance and observability workflows.

This package is design-spec focused (not implementation code).

## Final Prompt (Copy/Paste)

```text
You are a principal product designer + frontend UX architect.

Design a complete frontend UX/spec for **bominal** that is mobile-optimized and desktop-friendly, high-performance, and implementation-ready.

## Product context (must preserve)
- Brand/product name: `bominal`
- Entry URL: `www.bominal.com` (auth landing)
- Auth landing behavior:
  - Primary CTA: `Authenticate with passkey`
  - Secondary CTA: `Sign in with email/password`
- After successful auth, user goes to a dashboard.
- There is a separate **Admin App** (not visible to non-admin users) for maintenance and operations.
- Admin maintenance must integrate observability and comprehensive management tasks.
- Existing runtime observability surfaces include:
  - Health endpoint: `/health`
  - Readiness endpoint: `/ready`
  - Admin metrics endpoint: `/admin/maintenance/metrics`
  - Admin maintenance route: `/admin/maintenance`

## What to produce
Return a **Design Spec + Screen Blueprint** (not code) with these sections:

1. Information Architecture
- Site map for user app and separate admin app.
- Navigation model for mobile and desktop.
- Role-based visibility model: non-admin vs admin.

2. Core User Flows
- Passkey-first sign-in
- Email/password fallback
- Failed auth/recovery states
- Post-login dashboard onboarding
- Admin access entry and guardrails

3. Screen-by-screen Blueprint (mobile + desktop)
- Auth landing
- User dashboard (default)
- Key user dashboard detail views
- Admin app shell
- Admin maintenance dashboard
- Admin management modules (see scope below)

For each screen include:
- Layout structure
- Component inventory
- State variants (loading/empty/error/success)
- Accessibility notes
- Primary and secondary actions

4. Admin “Comprehensive Management” Scope
Design modules for:
- User/session/role management
- Auth/security controls (session revocation, access toggles)
- Runtime operations (queue/job controls, retry/requeue, kill switches)
- System status and dependency health
- Observability views (metrics, logs/events timeline, incident context)
- Configuration visibility (safe, redacted; no secret exposure)

5. Observability UX
- Operational summary cards (SLO-ish status, error rate, latency, saturation)
- Drill-down paths from high-level status to actionable panels
- Alert states and incident workflow
- “Support code/request ID” visibility for debugging
- Explicit distinction between health (`/health`) and readiness (`/ready`)
- Admin metrics viewer UX using `/admin/maintenance/metrics`

6. Safety + Security UX (strict guardrails)
- Admin-only visibility and route protection assumptions
- Step-up auth for sensitive actions
- Confirmation + typed confirmation for destructive actions
- Mandatory “reason for change” on high-risk operations
- Immutable audit log/event trail UX
- Fail-closed behavior patterns

7. Design System Direction
Style target: **Operational Premium**
- Trustworthy, clear, data-dense, low-fatigue visual system
- Strong hierarchy, readable typography, consistent spacing
- Works well on small mobile screens and wide desktop layouts
- Avoid generic “template” feeling; create intentional visual identity

Include:
- Token recommendations (color, spacing, type scale, radius, elevation)
- Component behavior rules (tables, filters, drawers, modals, toasts, badges)
- Motion guidance (subtle, purposeful, reduced-motion compatible)

8. Performance + Accessibility Requirements
- WCAG 2.2 AA baseline
- Keyboard-first admin usability
- Minimal layout shift, predictable focus order
- Responsive performance budgets and lightweight interaction patterns
- Prioritize SSR-friendly, progressively enhanced UI behavior

9. Acceptance Criteria
Provide measurable acceptance criteria per major flow and screen family.

## Constraints and policies
- Never expose secrets, tokens, raw payment payloads, or sensitive internals in UI.
- Admin tools must default safe and fail closed.
- Keep copy concise, high signal, and operationally clear.
- You may improve or replace parts of the initial requirements if you justify why the result is safer, clearer, or higher performing.

## Output format
Return in this exact structure:
- Executive summary
- IA map
- User app blueprint
- Admin app blueprint
- Admin maintenance + observability blueprint
- Component/tokens spec
- Accessibility/performance checklist
- Acceptance criteria matrix
- Open risks and recommended mitigations
```

## Defaults Locked

- Admin scope: Full ops suite.
- Admin access model: Separate admin app.
- Visual style: Operational premium.
- Safety model: Strict guardrails.
- Auth entry: Passkey-first with email/password fallback.

