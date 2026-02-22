"""add task list tail-latency indexes

Revision ID: 20260215_0009
Revises: 20260214_0008
Create Date: 2026-02-15 00:00:00.000000
"""

from typing import Sequence, Union

import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260215_0009"
down_revision: Union[str, None] = "20260214_0008"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_index(
        "ix_task_attempts_task_finished_id_desc",
        "task_attempts",
        ["task_id", sa.text("finished_at DESC"), sa.text("id DESC")],
        unique=False,
    )
    op.create_index(
        "ix_artifacts_task_kind_created_id_desc",
        "artifacts",
        ["task_id", "kind", sa.text("created_at DESC"), sa.text("id DESC")],
        unique=False,
    )
    op.create_index(
        "ix_tasks_train_active_user_created_desc",
        "tasks",
        ["user_id", sa.text("created_at DESC"), sa.text("id DESC")],
        unique=False,
        postgresql_where=sa.text(
            "module = 'train' AND hidden_at IS NULL AND state IN "
            "('QUEUED','RUNNING','POLLING','RESERVING','PAYING','PAUSED')"
        ),
    )
    op.create_index(
        "ix_tasks_train_terminal_user_created_desc",
        "tasks",
        ["user_id", sa.text("created_at DESC"), sa.text("id DESC")],
        unique=False,
        postgresql_where=sa.text(
            "module = 'train' AND hidden_at IS NULL AND state IN "
            "('COMPLETED','EXPIRED','CANCELLED','FAILED')"
        ),
    )


def downgrade() -> None:
    op.drop_index("ix_tasks_train_terminal_user_created_desc", table_name="tasks")
    op.drop_index("ix_tasks_train_active_user_created_desc", table_name="tasks")
    op.drop_index("ix_artifacts_task_kind_created_id_desc", table_name="artifacts")
    op.drop_index("ix_task_attempts_task_finished_id_desc", table_name="task_attempts")
