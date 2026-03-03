"""add user ui locale

Revision ID: 20260209_0007
Revises: 20260204_0006
Create Date: 2026-02-09 00:00:00.000000
"""

from typing import Sequence, Union

import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260209_0007"
down_revision: Union[str, None] = "20260204_0006"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.add_column(
        "users",
        sa.Column("ui_locale", sa.String(length=8), nullable=False, server_default=sa.text("'en'")),
    )


def downgrade() -> None:
    op.drop_column("users", "ui_locale")

