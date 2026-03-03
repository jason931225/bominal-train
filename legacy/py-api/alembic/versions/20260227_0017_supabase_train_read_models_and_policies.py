"""add supabase train read models and owner-select rls policies

Revision ID: 20260227_0017
Revises: 20260227_0016
Create Date: 2026-02-27 03:00:00.000000
"""

from __future__ import annotations

from typing import Sequence, Union

from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260227_0017"
down_revision: Union[str, None] = "20260227_0016"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.execute("ALTER TABLE tasks ENABLE ROW LEVEL SECURITY")
    op.execute("ALTER TABLE task_attempts ENABLE ROW LEVEL SECURITY")
    op.execute("ALTER TABLE artifacts ENABLE ROW LEVEL SECURITY")

    op.execute("DROP POLICY IF EXISTS tasks_select_own ON tasks")
    op.execute("DROP POLICY IF EXISTS task_attempts_select_own ON task_attempts")
    op.execute("DROP POLICY IF EXISTS artifacts_select_own ON artifacts")

    op.execute(
        """
        DO $$
        BEGIN
          IF EXISTS (
            SELECT 1
            FROM pg_proc p
            JOIN pg_namespace n ON n.oid = p.pronamespace
            WHERE n.nspname = 'auth'
              AND p.proname = 'uid'
          ) THEN
            CREATE POLICY tasks_select_own
            ON tasks
            FOR SELECT
            TO authenticated
            USING (
              EXISTS (
                SELECT 1
                FROM users
                WHERE users.id = tasks.user_id
                  AND users.supabase_user_id = auth.uid()::text
              )
            );

            CREATE POLICY task_attempts_select_own
            ON task_attempts
            FOR SELECT
            TO authenticated
            USING (
              EXISTS (
                SELECT 1
                FROM tasks
                JOIN users ON users.id = tasks.user_id
                WHERE tasks.id = task_attempts.task_id
                  AND users.supabase_user_id = auth.uid()::text
              )
            );

            CREATE POLICY artifacts_select_own
            ON artifacts
            FOR SELECT
            TO authenticated
            USING (
              EXISTS (
                SELECT 1
                FROM tasks
                JOIN users ON users.id = tasks.user_id
                WHERE tasks.id = artifacts.task_id
                  AND users.supabase_user_id = auth.uid()::text
              )
            );
          END IF;
        END
        $$;
        """
    )

    op.execute(
        """
        CREATE OR REPLACE VIEW v_train_task_list_compact AS
        WITH latest_attempt AS (
          SELECT DISTINCT ON (ta.task_id)
            ta.task_id,
            ta.finished_at AS last_attempt_at,
            ta.action AS last_attempt_action,
            ta.ok AS last_attempt_ok,
            ta.error_code AS last_attempt_error_code,
            ta.error_message_safe AS last_attempt_error_message_safe,
            ta.finished_at AS last_attempt_finished_at
          FROM task_attempts AS ta
          ORDER BY ta.task_id, ta.finished_at DESC, ta.id DESC
        ),
        task_base AS (
          SELECT
            t.*,
            NULLIF(BTRIM(COALESCE(t.spec_json->>'next_run_at', '')), '') AS next_run_at_text,
            NULLIF(BTRIM(COALESCE(t.spec_json->>'manual_retry_last_at', '')), '') AS manual_retry_last_at_text
          FROM tasks AS t
          WHERE t.module = 'train'
            AND t.hidden_at IS NULL
        )
        SELECT
          t.id,
          t.module,
          t.state,
          t.deadline_at,
          t.created_at,
          t.updated_at,
          t.paused_at,
          t.cancelled_at,
          t.completed_at,
          t.failed_at,
          t.hidden_at,
          la.last_attempt_at,
          la.last_attempt_action,
          la.last_attempt_ok,
          la.last_attempt_error_code,
          la.last_attempt_error_message_safe,
          la.last_attempt_finished_at,
          CASE
            WHEN t.state = 'POLLING'
             AND t.next_run_at_text IS NOT NULL
             AND t.next_run_at_text ~ '^\\d{4}-\\d{2}-\\d{2}T'
            THEN t.next_run_at_text::timestamptz
            ELSE NULL
          END AS next_run_at,
          CASE
            WHEN t.state = 'PAUSED' OR t.paused_at IS NOT NULL THEN FALSE
            WHEN t.state IN ('RUNNING', 'RESERVING', 'PAYING') THEN FALSE
            WHEN t.state NOT IN ('QUEUED', 'POLLING', 'EXPIRED', 'CANCELLED', 'FAILED') THEN FALSE
            WHEN NOW() >= t.deadline_at THEN FALSE
            WHEN t.manual_retry_last_at_text IS NULL OR t.manual_retry_last_at_text !~ '^\\d{4}-\\d{2}-\\d{2}T' THEN TRUE
            WHEN NOW() < ((t.manual_retry_last_at_text)::timestamptz + INTERVAL '15 second') THEN FALSE
            ELSE TRUE
          END AS retry_now_allowed,
          CASE
            WHEN t.state = 'PAUSED' OR t.paused_at IS NOT NULL THEN 'paused_use_resume'
            WHEN t.state IN ('RUNNING', 'RESERVING', 'PAYING') THEN 'task_running'
            WHEN t.state NOT IN ('QUEUED', 'POLLING', 'EXPIRED', 'CANCELLED', 'FAILED') THEN 'not_eligible_state'
            WHEN NOW() >= t.deadline_at THEN 'deadline_passed'
            WHEN t.manual_retry_last_at_text IS NOT NULL
             AND t.manual_retry_last_at_text ~ '^\\d{4}-\\d{2}-\\d{2}T'
             AND NOW() < ((t.manual_retry_last_at_text)::timestamptz + INTERVAL '15 second')
            THEN 'cooldown_active'
            ELSE NULL
          END AS retry_now_reason,
          CASE
            WHEN t.manual_retry_last_at_text IS NOT NULL
             AND t.manual_retry_last_at_text ~ '^\\d{4}-\\d{2}-\\d{2}T'
             AND NOW() < ((t.manual_retry_last_at_text)::timestamptz + INTERVAL '15 second')
            THEN ((t.manual_retry_last_at_text)::timestamptz + INTERVAL '15 second')
            ELSE NULL
          END AS retry_now_available_at,
          t.spec_json,
          tre.ticket_status,
          tre.ticket_paid,
          tre.ticket_payment_deadline_at,
          tre.ticket_reservation_id,
          tre.ticket_train_no,
          tre.ticket_seat_count,
          tre.ticket_seats,
          tre.ticket_seat_classes,
          COALESCE(
            tre.list_bucket,
            CASE
              WHEN t.state IN ('QUEUED', 'RUNNING', 'POLLING', 'RESERVING', 'PAYING', 'PAUSED') THEN 'active'
              WHEN t.state = 'COMPLETED'
               AND COALESCE(tre.ticket_status, '') IN ('awaiting_payment', 'reserved', 'waiting')
               AND tre.ticket_paid IS DISTINCT FROM TRUE
              THEN 'active'
              ELSE 'completed'
            END
          )::varchar(16) AS list_bucket
        FROM task_base AS t
        LEFT JOIN latest_attempt AS la ON la.task_id = t.id
        LEFT JOIN task_realtime_events AS tre ON tre.task_id = t.id
        """
    )

    op.execute(
        """
        DO $$
        BEGIN
          IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'authenticated') THEN
            GRANT SELECT ON tasks TO authenticated;
            GRANT SELECT ON task_attempts TO authenticated;
            GRANT SELECT ON artifacts TO authenticated;
            GRANT SELECT ON v_train_task_list_compact TO authenticated;
          END IF;
        END
        $$;
        """
    )


def downgrade() -> None:
    op.execute(
        """
        DO $$
        BEGIN
          IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'authenticated') THEN
            REVOKE SELECT ON v_train_task_list_compact FROM authenticated;
            REVOKE SELECT ON artifacts FROM authenticated;
            REVOKE SELECT ON task_attempts FROM authenticated;
            REVOKE SELECT ON tasks FROM authenticated;
          END IF;
        END
        $$;
        """
    )

    op.execute("DROP VIEW IF EXISTS v_train_task_list_compact")
    op.execute("DROP POLICY IF EXISTS artifacts_select_own ON artifacts")
    op.execute("DROP POLICY IF EXISTS task_attempts_select_own ON task_attempts")
    op.execute("DROP POLICY IF EXISTS tasks_select_own ON tasks")
