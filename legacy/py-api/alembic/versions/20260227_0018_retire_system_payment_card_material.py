"""retire legacy serverwide system payment card material

Revision ID: 20260227_0018
Revises: 20260227_0017
Create Date: 2026-02-27 19:20:00.000000
"""

from __future__ import annotations

from typing import Sequence, Union

from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260227_0018"
down_revision: Union[str, None] = "20260227_0017"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.execute(
        """
        UPDATE system_payment_settings
        SET ciphertext = NULL,
            nonce = NULL,
            wrapped_dek = NULL,
            dek_nonce = NULL,
            aad = NULL,
            kek_version = NULL
        """
    )


def downgrade() -> None:
    # Irreversible data sanitation migration.
    pass
