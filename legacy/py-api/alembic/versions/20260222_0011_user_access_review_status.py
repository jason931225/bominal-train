"""add user access review status

Revision ID: 20260222_0011
Revises: 20260222_0010
Create Date: 2026-02-22 00:00:00.000000
"""

from typing import Sequence, Union

import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260222_0011"
down_revision: Union[str, None] = "20260222_0010"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    existing_columns = {column["name"] for column in inspector.get_columns("users")}
    existing_indexes = {index["name"] for index in inspector.get_indexes("users")}

    if "access_status" not in existing_columns:
        op.add_column(
            "users",
            sa.Column("access_status", sa.String(length=16), nullable=False, server_default="pending"),
        )
    if "access_reviewed_at" not in existing_columns:
        op.add_column(
            "users",
            sa.Column("access_reviewed_at", sa.DateTime(timezone=True), nullable=True),
        )
    if "ix_users_access_status" not in existing_indexes:
        op.create_index("ix_users_access_status", "users", ["access_status"], unique=False)

    # Preserve current access for existing users; new registrations remain pending by default.
    op.execute("UPDATE users SET access_status = 'approved', access_reviewed_at = NOW() WHERE access_status = 'pending'")


def downgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    existing_columns = {column["name"] for column in inspector.get_columns("users")}
    existing_indexes = {index["name"] for index in inspector.get_indexes("users")}

    if "ix_users_access_status" in existing_indexes:
        op.drop_index("ix_users_access_status", table_name="users")
    if "access_reviewed_at" in existing_columns:
        op.drop_column("users", "access_reviewed_at")
    if "access_status" in existing_columns:
        op.drop_column("users", "access_status")
