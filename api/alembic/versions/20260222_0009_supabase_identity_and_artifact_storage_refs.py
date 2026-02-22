"""add supabase identity and artifact storage reference columns

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
    op.add_column("users", sa.Column("supabase_user_id", sa.String(length=64), nullable=True))
    op.create_index("ix_users_supabase_user_id", "users", ["supabase_user_id"], unique=True)

    op.add_column("artifacts", sa.Column("storage_provider", sa.String(length=32), nullable=True))
    op.add_column("artifacts", sa.Column("storage_bucket", sa.String(length=128), nullable=True))
    op.add_column("artifacts", sa.Column("storage_object_path", sa.String(length=1024), nullable=True))
    op.add_column("artifacts", sa.Column("storage_content_type", sa.String(length=128), nullable=True))
    op.add_column("artifacts", sa.Column("storage_size_bytes", sa.Integer(), nullable=True))
    op.add_column("artifacts", sa.Column("storage_checksum_sha256", sa.String(length=128), nullable=True))


def downgrade() -> None:
    op.drop_column("artifacts", "storage_checksum_sha256")
    op.drop_column("artifacts", "storage_size_bytes")
    op.drop_column("artifacts", "storage_content_type")
    op.drop_column("artifacts", "storage_object_path")
    op.drop_column("artifacts", "storage_bucket")
    op.drop_column("artifacts", "storage_provider")

    op.drop_index("ix_users_supabase_user_id", table_name="users")
    op.drop_column("users", "supabase_user_id")
