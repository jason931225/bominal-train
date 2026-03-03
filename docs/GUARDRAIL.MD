# Guardrail

This repository treats `third_party/**` as read-only.

Non-negotiable rules:
- Do not create, edit, move, or delete files under `third_party/**`.
- In particular, `third_party/srtgo/**` and `third_party/catchtable/**` must remain unchanged.
- If behavior changes are needed, implement them outside `third_party/**`.
