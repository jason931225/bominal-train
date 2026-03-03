"""add optional user account profile fields

Revision ID: 20260203_0004
Revises: 20260203_0003
Create Date: 2026-02-03 10:30:00.000000
"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


# revision identifiers, used by Alembic.
revision: str = "20260203_0004"
down_revision: Union[str, None] = "20260203_0003"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column("users", sa.Column("phone_number", sa.String(length=32), nullable=True))
    op.add_column("users", sa.Column("billing_address", sa.Text(), nullable=True))
    op.add_column("users", sa.Column("birthday", sa.Date(), nullable=True))


def downgrade() -> None:
    op.drop_column("users", "birthday")
    op.drop_column("users", "billing_address")
    op.drop_column("users", "phone_number")

