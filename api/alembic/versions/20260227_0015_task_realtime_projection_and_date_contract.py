"""add task realtime projection and task date contract guard

Revision ID: 20260227_0015
Revises: 20260226_0014
Create Date: 2026-02-27 00:00:00.000000
"""

from __future__ import annotations

from typing import Sequence, Union

from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260227_0015"
down_revision: Union[str, None] = "20260226_0014"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Normalize historical compact task date strings (YYYYMMDD -> YYYY-MM-DD)
    # so task cards and downstream realtime projections rely on one canonical shape.
    op.execute(
        """
        UPDATE tasks
        SET spec_json = jsonb_set(
            spec_json,
            '{date}',
            to_jsonb(regexp_replace(spec_json->>'date', '^(\\d{4})(\\d{2})(\\d{2})$', '\\1-\\2-\\3')),
            true
        )
        WHERE spec_json ? 'date'
          AND jsonb_typeof(spec_json->'date') = 'string'
          AND (spec_json->>'date') ~ '^\\d{8}$'
        """
    )

    op.execute(
        """
        DO $$
        BEGIN
          IF NOT EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'ck_tasks_spec_date_iso_yyyy_mm_dd'
          ) THEN
            ALTER TABLE tasks
            ADD CONSTRAINT ck_tasks_spec_date_iso_yyyy_mm_dd
            CHECK (
              NOT (spec_json ? 'date')
              OR (
                jsonb_typeof(spec_json->'date') = 'string'
                AND (spec_json->>'date') ~ '^\\d{4}-\\d{2}-\\d{2}$'
              )
            );
          END IF;
        END
        $$;
        """
    )

    op.execute(
        """
        CREATE TABLE IF NOT EXISTS task_realtime_events (
          task_id UUID PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
          user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
          module VARCHAR(32) NOT NULL,
          state VARCHAR(32) NOT NULL,
          spec_date DATE NULL,
          updated_at TIMESTAMPTZ NOT NULL
        )
        """
    )
    op.execute(
        """
        CREATE INDEX IF NOT EXISTS ix_task_realtime_events_user_updated_desc
        ON task_realtime_events (user_id, updated_at DESC, task_id DESC)
        """
    )
    op.execute(
        """
        CREATE INDEX IF NOT EXISTS ix_task_realtime_events_state_updated_desc
        ON task_realtime_events (state, updated_at DESC, task_id DESC)
        """
    )

    op.execute(
        """
        INSERT INTO task_realtime_events (task_id, user_id, module, state, spec_date, updated_at)
        SELECT
          t.id,
          t.user_id,
          t.module,
          t.state,
          CASE
            WHEN (t.spec_json->>'date') ~ '^\\d{4}-\\d{2}-\\d{2}$' THEN (t.spec_json->>'date')::date
            ELSE NULL
          END AS spec_date,
          t.updated_at
        FROM tasks AS t
        WHERE t.module = 'train'
        ON CONFLICT (task_id) DO UPDATE
        SET
          user_id = EXCLUDED.user_id,
          module = EXCLUDED.module,
          state = EXCLUDED.state,
          spec_date = EXCLUDED.spec_date,
          updated_at = EXCLUDED.updated_at
        """
    )

    op.execute(
        """
        CREATE OR REPLACE FUNCTION sync_task_realtime_events()
        RETURNS trigger
        LANGUAGE plpgsql
        AS $$
        DECLARE
          v_date_text text;
          v_spec_date date;
        BEGIN
          IF TG_OP = 'DELETE' THEN
            DELETE FROM task_realtime_events WHERE task_id = OLD.id;
            RETURN OLD;
          END IF;

          IF NEW.module <> 'train' THEN
            DELETE FROM task_realtime_events WHERE task_id = NEW.id;
            RETURN NEW;
          END IF;

          v_date_text := NULLIF(COALESCE(NEW.spec_json->>'date', ''), '');
          IF v_date_text ~ '^\\d{4}-\\d{2}-\\d{2}$' THEN
            v_spec_date := v_date_text::date;
          ELSE
            v_spec_date := NULL;
          END IF;

          INSERT INTO task_realtime_events (task_id, user_id, module, state, spec_date, updated_at)
          VALUES (NEW.id, NEW.user_id, NEW.module, NEW.state, v_spec_date, NEW.updated_at)
          ON CONFLICT (task_id) DO UPDATE
          SET
            user_id = EXCLUDED.user_id,
            module = EXCLUDED.module,
            state = EXCLUDED.state,
            spec_date = EXCLUDED.spec_date,
            updated_at = EXCLUDED.updated_at;

          RETURN NEW;
        END
        $$;
        """
    )

    op.execute("DROP TRIGGER IF EXISTS trg_sync_task_realtime_events_upsert ON tasks")
    op.execute("DROP TRIGGER IF EXISTS trg_sync_task_realtime_events_delete ON tasks")
    op.execute(
        """
        CREATE TRIGGER trg_sync_task_realtime_events_upsert
        AFTER INSERT OR UPDATE OF user_id, module, state, spec_json, updated_at ON tasks
        FOR EACH ROW
        EXECUTE FUNCTION sync_task_realtime_events()
        """
    )
    op.execute(
        """
        CREATE TRIGGER trg_sync_task_realtime_events_delete
        AFTER DELETE ON tasks
        FOR EACH ROW
        EXECUTE FUNCTION sync_task_realtime_events()
        """
    )

    op.execute("ALTER TABLE task_realtime_events ENABLE ROW LEVEL SECURITY")
    op.execute("DROP POLICY IF EXISTS task_realtime_events_select_own ON task_realtime_events")
    op.execute("DROP POLICY IF EXISTS task_realtime_events_deny_all ON task_realtime_events")
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
            CREATE POLICY task_realtime_events_select_own
            ON task_realtime_events
            FOR SELECT
            TO authenticated
            USING (
              EXISTS (
                SELECT 1
                FROM users
                WHERE users.id = task_realtime_events.user_id
                  AND users.supabase_user_id = auth.uid()::text
              )
            );
          ELSE
            CREATE POLICY task_realtime_events_deny_all
            ON task_realtime_events
            AS PERMISSIVE
            FOR ALL
            TO PUBLIC
            USING (false)
            WITH CHECK (false);
          END IF;
        END
        $$;
        """
    )

    op.execute(
        """
        DO $$
        BEGIN
          IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'authenticated') THEN
            GRANT SELECT ON task_realtime_events TO authenticated;
          END IF;
        END
        $$;
        """
    )

    op.execute(
        """
        DO $$
        BEGIN
          BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'bominal_realtime') THEN
              CREATE PUBLICATION bominal_realtime;
            END IF;

            IF NOT EXISTS (
              SELECT 1
              FROM pg_publication_tables
              WHERE pubname = 'bominal_realtime'
                AND schemaname = 'public'
                AND tablename = 'task_realtime_events'
            ) THEN
              ALTER PUBLICATION bominal_realtime ADD TABLE public.task_realtime_events;
            END IF;
          EXCEPTION
            WHEN insufficient_privilege THEN
              RAISE NOTICE 'Skipping publication updates due to insufficient privilege';
          END;
        END
        $$;
        """
    )


def downgrade() -> None:
    op.execute(
        """
        DO $$
        BEGIN
          BEGIN
            IF EXISTS (
              SELECT 1
              FROM pg_publication_tables
              WHERE pubname = 'bominal_realtime'
                AND schemaname = 'public'
                AND tablename = 'task_realtime_events'
            ) THEN
              ALTER PUBLICATION bominal_realtime DROP TABLE public.task_realtime_events;
            END IF;
          EXCEPTION
            WHEN undefined_object OR insufficient_privilege THEN
              RAISE NOTICE 'Skipping publication rollback due to missing publication or insufficient privilege';
          END;
        END
        $$;
        """
    )

    op.execute("DROP TRIGGER IF EXISTS trg_sync_task_realtime_events_delete ON tasks")
    op.execute("DROP TRIGGER IF EXISTS trg_sync_task_realtime_events_upsert ON tasks")
    op.execute("DROP FUNCTION IF EXISTS sync_task_realtime_events")

    op.execute("DROP POLICY IF EXISTS task_realtime_events_select_own ON task_realtime_events")
    op.execute("DROP POLICY IF EXISTS task_realtime_events_deny_all ON task_realtime_events")
    op.execute("DROP TABLE IF EXISTS task_realtime_events")
    op.execute("ALTER TABLE tasks DROP CONSTRAINT IF EXISTS ck_tasks_spec_date_iso_yyyy_mm_dd")
