# Phase 4: Auth Pages - Discussion Log

**Mode:** Smart discuss
**Date:** 2026-03-27

No user questionnaire was required because the route inventory is already fixed and the repo contains both partial `bominal-app` auth pages and full donor references.

Key carry-forward constraints:
- Keep the Phase 3 route table unchanged.
- Reuse `crate::api` and `crate::state` rather than donor direct-service auth calls.
- Pull passkey interop forward only where the auth pages need it; do not broaden scope into the full Phase 7 interop backlog.
