# Phase 2: Core Infrastructure - Discussion Log

**Mode:** Smart discuss
**Date:** 2026-03-27

Phase 2 was treated as infrastructure-only. No user-facing grey-area questionnaire was required.

Key carry-forward constraints:
- `bominal-app` remains the active migration target.
- `bominal-frontend` remains donor/reference code only.
- App-side server functions proxy existing `/api/` endpoints.
- Shared domain/i18n data should come from canonical crates where possible.
