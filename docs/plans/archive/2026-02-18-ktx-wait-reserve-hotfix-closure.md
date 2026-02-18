# KTX Wait-Reserve Hotfix Closure (2026-02-18)

## Summary

- Root cause confirmed on branch head `b70af72`: worker SEARCH candidate selection only treated direct-seat availability (`availability.general/special`) as reservable.
- Waitlist-capable KTX schedules (`metadata.wait_reserve_flag >= 0`) were filtered out before reserve, causing retryable `seat_unavailable`.
- Hotfix implemented and verified on `feat-stage10-tasklist-tail-latency`.

## Implemented Changes

1. Worker seat selection is now schedule-aware:
   - direct seat availability remains first priority.
   - wait-reserve fallback is enabled when provider metadata indicates standby support.
2. Provider parity logic:
   - KTX standby eligibility: `wait_reserve_flag >= 0`
   - SRT standby eligibility: `reserve_wait_code >= 0`
3. Standby seat-class behavior:
   - `special` / `special_preferred` -> reserve with `special`
   - otherwise -> reserve with `general`
4. Added regression tests:
   - KTX waitlist-only schedule path proceeds to reserve and completes.
   - SRT waitlist-only parity path reserves with `special` under `special_preferred`.

## Verification Evidence

- `docker compose -f infra/docker-compose.yml exec -T api pytest -q tests/test_train_tasks.py -k 'wait_reserve_path_runs_when_selected_train_is_waitlist_only or srt_wait_reserve_path_uses_special_for_special_preferred or seat_preference_fallback_reserves_available_class'`
  - Result: `4 passed`
- `docker compose -f infra/docker-compose.yml exec -T api pytest -q tests/test_train_tasks.py`
  - Result: `36 passed`

## Commits

- `d28cb77` - lock acquire
- `388ae4e` - worker fix + regression tests
- `86c6a14` - changelog entry
- `4c7744d` - lock release

## Rollback

1. Revert behavior commit:
   - `git revert 388ae4e`
2. If needed, revert changelog follow-up:
   - `git revert 86c6a14`
3. Re-run train task tests before deploy:
   - `docker compose -f infra/docker-compose.yml exec -T api pytest -q tests/test_train_tasks.py`
