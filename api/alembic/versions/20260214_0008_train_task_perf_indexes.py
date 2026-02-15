"""add train task performance indexes

Revision ID: 20260214_0008
Revises: 20260209_0007
Create Date: 2026-02-14 00:00:00.000000
"""

from typing import Sequence, Union

from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260214_0008"
down_revision: Union[str, None] = "20260209_0007"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_index(
        "ix_tasks_user_module_hidden_state_created_at",
        "tasks",
        ["user_id", "module", "hidden_at", "state", "created_at"],
        unique=False,
    )
    op.create_index(
        "ix_task_attempts_task_finished_at",
        "task_attempts",
        ["task_id", "finished_at"],
        unique=False,
    )
    op.create_index(
        "ix_artifacts_task_kind_created_at",
        "artifacts",
        ["task_id", "kind", "created_at"],
        unique=False,
    )


def downgrade() -> None:
    op.drop_index("ix_artifacts_task_kind_created_at", table_name="artifacts")
    op.drop_index("ix_task_attempts_task_finished_at", table_name="task_attempts")
    op.drop_index("ix_tasks_user_module_hidden_state_created_at", table_name="tasks")
