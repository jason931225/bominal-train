"""add soft-hide column for tasks

Revision ID: 20260203_0003
Revises: 20260203_0002
Create Date: 2026-02-03 06:20:00.000000
"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


# revision identifiers, used by Alembic.
revision: str = "20260203_0003"
down_revision: Union[str, None] = "20260203_0002"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column("tasks", sa.Column("hidden_at", sa.DateTime(timezone=True), nullable=True))
    op.create_index("ix_tasks_hidden_at", "tasks", ["hidden_at"], unique=False)


def downgrade() -> None:
    op.drop_index("ix_tasks_hidden_at", table_name="tasks")
    op.drop_column("tasks", "hidden_at")
