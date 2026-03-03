"""add task engine tables

Revision ID: 20260203_0002
Revises: 20260203_0001
Create Date: 2026-02-03 00:30:00.000000
"""

from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql


# revision identifiers, used by Alembic.
revision: str = "20260203_0002"
down_revision: Union[str, None] = "20260203_0001"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.create_table(
        "tasks",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True, nullable=False),
        sa.Column("user_id", postgresql.UUID(as_uuid=True), sa.ForeignKey("users.id"), nullable=False),
        sa.Column("module", sa.String(length=32), nullable=False),
        sa.Column("state", sa.String(length=32), nullable=False),
        sa.Column("deadline_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("spec_json", postgresql.JSONB(astext_type=sa.Text()), nullable=False),
        sa.Column("idempotency_key", sa.String(length=64), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.text("CURRENT_TIMESTAMP"), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.text("CURRENT_TIMESTAMP"), nullable=False),
        sa.Column("paused_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("cancelled_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("completed_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("failed_at", sa.DateTime(timezone=True), nullable=True),
    )
    op.create_index("ix_tasks_user_id", "tasks", ["user_id"], unique=False)
    op.create_index("ix_tasks_module", "tasks", ["module"], unique=False)
    op.create_index("ix_tasks_state", "tasks", ["state"], unique=False)
    op.create_index("ix_tasks_deadline_at", "tasks", ["deadline_at"], unique=False)
    op.create_index("ix_tasks_idempotency_key", "tasks", ["idempotency_key"], unique=False)

    op.execute(
        sa.text(
            """
            CREATE UNIQUE INDEX uq_tasks_active_idempotency
            ON tasks (user_id, module, idempotency_key)
            WHERE state NOT IN ('COMPLETED', 'EXPIRED', 'CANCELLED', 'FAILED')
            """
        )
    )

    op.create_table(
        "task_attempts",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True, nullable=False),
        sa.Column("task_id", postgresql.UUID(as_uuid=True), sa.ForeignKey("tasks.id"), nullable=False),
        sa.Column("action", sa.String(length=16), nullable=False),
        sa.Column("provider", sa.String(length=8), nullable=False),
        sa.Column("ok", sa.Boolean(), nullable=False),
        sa.Column("retryable", sa.Boolean(), nullable=False),
        sa.Column("error_code", sa.String(length=64), nullable=True),
        sa.Column("error_message_safe", sa.Text(), nullable=True),
        sa.Column("duration_ms", sa.Integer(), nullable=False),
        sa.Column("meta_json_safe", postgresql.JSONB(astext_type=sa.Text()), nullable=True),
        sa.Column("started_at", sa.DateTime(timezone=True), server_default=sa.text("CURRENT_TIMESTAMP"), nullable=False),
        sa.Column("finished_at", sa.DateTime(timezone=True), server_default=sa.text("CURRENT_TIMESTAMP"), nullable=False),
    )
    op.create_index("ix_task_attempts_task_id", "task_attempts", ["task_id"], unique=False)
    op.create_index("ix_task_attempts_provider", "task_attempts", ["provider"], unique=False)

    op.create_table(
        "artifacts",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True, nullable=False),
        sa.Column("task_id", postgresql.UUID(as_uuid=True), sa.ForeignKey("tasks.id"), nullable=False),
        sa.Column("module", sa.String(length=32), nullable=False),
        sa.Column("kind", sa.String(length=32), nullable=False),
        sa.Column("data_json_safe", postgresql.JSONB(astext_type=sa.Text()), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.text("CURRENT_TIMESTAMP"), nullable=False),
    )
    op.create_index("ix_artifacts_task_id", "artifacts", ["task_id"], unique=False)
    op.create_index("ix_artifacts_module", "artifacts", ["module"], unique=False)
    op.create_index("ix_artifacts_kind", "artifacts", ["kind"], unique=False)

    op.create_table(
        "secrets",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True, nullable=False),
        sa.Column("user_id", postgresql.UUID(as_uuid=True), sa.ForeignKey("users.id"), nullable=False),
        sa.Column("kind", sa.String(length=64), nullable=False),
        sa.Column("ciphertext", sa.Text(), nullable=False),
        sa.Column("nonce", sa.String(length=128), nullable=False),
        sa.Column("wrapped_dek", sa.Text(), nullable=False),
        sa.Column("dek_nonce", sa.String(length=128), nullable=False),
        sa.Column("aad", sa.Text(), nullable=False),
        sa.Column("kek_version", sa.Integer(), nullable=False),
        sa.Column("created_at", sa.DateTime(timezone=True), server_default=sa.text("CURRENT_TIMESTAMP"), nullable=False),
        sa.Column("updated_at", sa.DateTime(timezone=True), server_default=sa.text("CURRENT_TIMESTAMP"), nullable=False),
    )
    op.create_index("ix_secrets_user_id", "secrets", ["user_id"], unique=False)
    op.create_index("ix_secrets_kind", "secrets", ["kind"], unique=False)


def downgrade() -> None:
    op.drop_index("ix_secrets_kind", table_name="secrets")
    op.drop_index("ix_secrets_user_id", table_name="secrets")
    op.drop_table("secrets")

    op.drop_index("ix_artifacts_kind", table_name="artifacts")
    op.drop_index("ix_artifacts_module", table_name="artifacts")
    op.drop_index("ix_artifacts_task_id", table_name="artifacts")
    op.drop_table("artifacts")

    op.drop_index("ix_task_attempts_provider", table_name="task_attempts")
    op.drop_index("ix_task_attempts_task_id", table_name="task_attempts")
    op.drop_table("task_attempts")

    op.execute(sa.text("DROP INDEX IF EXISTS uq_tasks_active_idempotency"))
    op.drop_index("ix_tasks_idempotency_key", table_name="tasks")
    op.drop_index("ix_tasks_deadline_at", table_name="tasks")
    op.drop_index("ix_tasks_state", table_name="tasks")
    op.drop_index("ix_tasks_module", table_name="tasks")
    op.drop_index("ix_tasks_user_id", table_name="tasks")
    op.drop_table("tasks")
