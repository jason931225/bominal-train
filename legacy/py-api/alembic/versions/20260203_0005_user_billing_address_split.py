"""split billing address into structured fields

Revision ID: 20260203_0005
Revises: 20260203_0004
Create Date: 2026-02-03 18:40:00.000000
"""

from typing import Any, Sequence, Union

import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260203_0005"
down_revision: Union[str, None] = "20260203_0004"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    columns = {column["name"] for column in inspector.get_columns("users")}

    additions: list[tuple[str, sa.Column[Any]]] = [
        ("billing_address_line1", sa.Column("billing_address_line1", sa.String(length=255), nullable=True)),
        ("billing_address_line2", sa.Column("billing_address_line2", sa.String(length=255), nullable=True)),
        ("billing_city", sa.Column("billing_city", sa.String(length=128), nullable=True)),
        (
            "billing_state_province",
            sa.Column("billing_state_province", sa.String(length=128), nullable=True),
        ),
        ("billing_country", sa.Column("billing_country", sa.String(length=128), nullable=True)),
        ("billing_postal_code", sa.Column("billing_postal_code", sa.String(length=32), nullable=True)),
    ]

    for name, column in additions:
        if name not in columns:
            op.add_column("users", column)


def downgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    columns = {column["name"] for column in inspector.get_columns("users")}

    for name in (
        "billing_postal_code",
        "billing_country",
        "billing_state_province",
        "billing_city",
        "billing_address_line2",
        "billing_address_line1",
    ):
        if name in columns:
            op.drop_column("users", name)
