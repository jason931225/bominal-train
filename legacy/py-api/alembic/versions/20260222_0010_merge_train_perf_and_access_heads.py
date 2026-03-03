"""merge train perf and access review heads

Revision ID: 20260222_0010
Revises: 20260215_0009, 20260222_0009
Create Date: 2026-02-22 00:55:00.000000
"""

from typing import Sequence, Union


# revision identifiers, used by Alembic.
revision: str = "20260222_0010"
down_revision: Union[str, tuple[str, str], None] = ("20260215_0009", "20260222_0009")
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Merge revision only; no schema changes.
    pass


def downgrade() -> None:
    # No-op because this revision only merges heads.
    pass
