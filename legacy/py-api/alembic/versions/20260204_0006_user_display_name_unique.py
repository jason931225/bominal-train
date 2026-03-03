"""enforce unique display name

Revision ID: 20260204_0006
Revises: 20260203_0005
Create Date: 2026-02-04 00:05:00.000000
"""

from typing import Sequence, Union

import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260204_0006"
down_revision: Union[str, None] = "20260203_0005"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

UNIQUE_CONSTRAINT_NAME = "uq_users_display_name"


def upgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    unique_constraints = {constraint["name"] for constraint in inspector.get_unique_constraints("users")}

    if UNIQUE_CONSTRAINT_NAME not in unique_constraints:
        op.create_unique_constraint(UNIQUE_CONSTRAINT_NAME, "users", ["display_name"])


def downgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    unique_constraints = {constraint["name"] for constraint in inspector.get_unique_constraints("users")}

    if UNIQUE_CONSTRAINT_NAME in unique_constraints:
        op.drop_constraint(UNIQUE_CONSTRAINT_NAME, "users", type_="unique")
