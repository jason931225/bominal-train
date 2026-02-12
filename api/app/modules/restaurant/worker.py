from __future__ import annotations

import logging

logger = logging.getLogger(__name__)


async def run_restaurant_task(ctx: dict, task_id: str) -> None:
    """Placeholder restaurant worker entrypoint.

    Stage 2 wires isolated worker processes before restaurant task execution
    is implemented. This function intentionally does not mutate state.
    """
    logger.info("Restaurant worker received task %s (not implemented yet)", task_id)

