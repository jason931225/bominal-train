"""add user access review status

Revision ID: 20260222_0009
Revises: 20260214_0008
Create Date: 2026-02-22 00:00:00.000000
"""

from typing import Sequence, Union

import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260222_0009"
down_revision: Union[str, None] = "20260214_0008"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column(
        "users",
        sa.Column("access_status", sa.String(length=16), nullable=False, server_default="pending"),
    )
    op.add_column(
        "users",
        sa.Column("access_reviewed_at", sa.DateTime(timezone=True), nullable=True),
    )
    op.create_index("ix_users_access_status", "users", ["access_status"], unique=False)
    # Preserve current access for existing users; new registrations remain pending by default.
    op.execute("UPDATE users SET access_status = 'approved', access_reviewed_at = NOW() WHERE access_status = 'pending'")


def downgrade() -> None:
    op.drop_index("ix_users_access_status", table_name="users")
    op.drop_column("users", "access_reviewed_at")
    op.drop_column("users", "access_status")
