"""add system payment settings table

Revision ID: 20260226_0014
Revises: 20260225_0013
Create Date: 2026-02-26 00:00:00.000000
"""

from __future__ import annotations

from typing import Sequence, Union

import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260226_0014"
down_revision: Union[str, None] = "20260225_0013"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "system_payment_settings",
        sa.Column("id", sa.Integer(), nullable=False),
        sa.Column("payment_enabled", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column("ciphertext", sa.Text(), nullable=True),
        sa.Column("nonce", sa.String(length=128), nullable=True),
        sa.Column("wrapped_dek", sa.Text(), nullable=True),
        sa.Column("dek_nonce", sa.String(length=128), nullable=True),
        sa.Column("aad", sa.Text(), nullable=True),
        sa.Column("kek_version", sa.Integer(), nullable=True),
        sa.Column("updated_by_user_id", sa.Uuid(), nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.text("now()")),
        sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False, server_default=sa.text("now()")),
        sa.ForeignKeyConstraint(["updated_by_user_id"], ["users.id"]),
        sa.PrimaryKeyConstraint("id"),
    )
    op.create_index(
        "ix_system_payment_settings_updated_by_user_id",
        "system_payment_settings",
        ["updated_by_user_id"],
        unique=False,
    )


def downgrade() -> None:
    op.drop_index("ix_system_payment_settings_updated_by_user_id", table_name="system_payment_settings")
    op.drop_table("system_payment_settings")
