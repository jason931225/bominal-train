"""add task list tail-latency indexes

Revision ID: 20260215_0009
Revises: 20260214_0008
Create Date: 2026-02-15 00:00:00.000000
"""

from typing import Sequence, Union

from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260215_0009"
down_revision: Union[str, None] = "20260214_0008"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Keep migration idempotent for databases that already contain these indexes.
    op.execute(
        """
        CREATE INDEX IF NOT EXISTS ix_task_attempts_task_finished_id_desc
        ON task_attempts (task_id, finished_at DESC, id DESC)
        """
    )
    op.execute(
        """
        CREATE INDEX IF NOT EXISTS ix_artifacts_task_kind_created_id_desc
        ON artifacts (task_id, kind, created_at DESC, id DESC)
        """
    )
    op.execute(
        """
        CREATE INDEX IF NOT EXISTS ix_tasks_train_active_user_created_desc
        ON tasks (user_id, created_at DESC, id DESC)
        WHERE module = 'train'
          AND hidden_at IS NULL
          AND state IN ('QUEUED','RUNNING','POLLING','RESERVING','PAYING','PAUSED')
        """
    )
    op.execute(
        """
        CREATE INDEX IF NOT EXISTS ix_tasks_train_terminal_user_created_desc
        ON tasks (user_id, created_at DESC, id DESC)
        WHERE module = 'train'
          AND hidden_at IS NULL
          AND state IN ('COMPLETED','EXPIRED','CANCELLED','FAILED')
        """
    )


def downgrade() -> None:
    op.execute("DROP INDEX IF EXISTS ix_tasks_train_terminal_user_created_desc")
    op.execute("DROP INDEX IF EXISTS ix_tasks_train_active_user_created_desc")
    op.execute("DROP INDEX IF EXISTS ix_artifacts_task_kind_created_id_desc")
    op.execute("DROP INDEX IF EXISTS ix_task_attempts_task_finished_id_desc")
