"""optimize task realtime projection for meaningful deltas

Revision ID: 20260227_0016
Revises: 20260227_0015
Create Date: 2026-02-27 00:30:00.000000
"""

from __future__ import annotations

from typing import Sequence, Union

from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260227_0016"
down_revision: Union[str, None] = "20260227_0015"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.execute(
        """
        ALTER TABLE task_realtime_events
        ADD COLUMN IF NOT EXISTS ticket_status VARCHAR(32),
        ADD COLUMN IF NOT EXISTS ticket_paid BOOLEAN,
        ADD COLUMN IF NOT EXISTS ticket_payment_deadline_at TIMESTAMPTZ,
        ADD COLUMN IF NOT EXISTS ticket_reservation_id VARCHAR(128),
        ADD COLUMN IF NOT EXISTS ticket_train_no VARCHAR(64),
        ADD COLUMN IF NOT EXISTS ticket_seat_count INTEGER,
        ADD COLUMN IF NOT EXISTS ticket_seats JSONB,
        ADD COLUMN IF NOT EXISTS ticket_seat_classes JSONB,
        ADD COLUMN IF NOT EXISTS list_bucket VARCHAR(16) NOT NULL DEFAULT 'active'
        """
    )

    op.execute(
        """
        DO $$
        BEGIN
          IF NOT EXISTS (
            SELECT 1
            FROM pg_constraint
            WHERE conname = 'ck_task_realtime_events_list_bucket'
          ) THEN
            ALTER TABLE task_realtime_events
            ADD CONSTRAINT ck_task_realtime_events_list_bucket
            CHECK (list_bucket IN ('active', 'completed'));
          END IF;
        END
        $$;
        """
    )

    op.execute(
        """
        CREATE OR REPLACE FUNCTION upsert_task_realtime_event_for_task(v_task_id UUID)
        RETURNS VOID
        LANGUAGE plpgsql
        AS $$
        DECLARE
          v_task RECORD;
          v_ticket JSONB;
          v_date_text TEXT;
          v_spec_date DATE;
          v_ticket_status TEXT;
          v_ticket_paid BOOLEAN;
          v_ticket_payment_deadline_at TIMESTAMPTZ;
          v_ticket_reservation_id TEXT;
          v_ticket_train_no TEXT;
          v_ticket_seat_count INTEGER;
          v_ticket_seats JSONB;
          v_ticket_seat_classes JSONB;
          v_deadline_text TEXT;
          v_projection_updated_at TIMESTAMPTZ;
          v_list_bucket TEXT;
        BEGIN
          SELECT
            t.id,
            t.user_id,
            t.module,
            t.state,
            t.spec_json,
            t.updated_at
          INTO v_task
          FROM tasks AS t
          WHERE t.id = v_task_id;

          IF NOT FOUND OR v_task.module <> 'train' THEN
            DELETE FROM task_realtime_events WHERE task_id = v_task_id;
            RETURN;
          END IF;

          v_date_text := NULLIF(COALESCE(v_task.spec_json->>'date', ''), '');
          IF v_date_text ~ '^\\d{4}-\\d{2}-\\d{2}$' THEN
            v_spec_date := v_date_text::DATE;
          ELSE
            v_spec_date := NULL;
          END IF;

          SELECT a.data_json_safe
          INTO v_ticket
          FROM artifacts AS a
          WHERE a.task_id = v_task_id
            AND a.module = 'train'
            AND a.kind = 'ticket'
          ORDER BY a.created_at DESC, a.id DESC
          LIMIT 1;

          v_ticket_status := NULL;
          v_ticket_paid := NULL;
          v_ticket_payment_deadline_at := NULL;
          v_ticket_reservation_id := NULL;
          v_ticket_train_no := NULL;
          v_ticket_seat_count := NULL;
          v_ticket_seats := NULL;
          v_ticket_seat_classes := NULL;

          IF v_ticket IS NOT NULL THEN
            v_ticket_status := NULLIF(BTRIM(COALESCE(v_ticket->>'status', '')), '');

            IF jsonb_typeof(v_ticket->'paid') = 'boolean' THEN
              v_ticket_paid := (v_ticket->>'paid')::BOOLEAN;
            END IF;

            v_deadline_text := NULLIF(BTRIM(COALESCE(v_ticket->>'payment_deadline_at', '')), '');
            IF v_deadline_text IS NOT NULL THEN
              BEGIN
                v_ticket_payment_deadline_at := v_deadline_text::TIMESTAMPTZ;
              EXCEPTION
                WHEN others THEN
                  v_ticket_payment_deadline_at := NULL;
              END;
            END IF;

            v_ticket_reservation_id := NULLIF(BTRIM(COALESCE(v_ticket->>'reservation_id', '')), '');

            v_ticket_train_no := NULLIF(BTRIM(COALESCE(v_ticket->>'train_no', '')), '');
            IF v_ticket_train_no IS NULL
              AND jsonb_typeof(v_ticket->'reservation_snapshot') = 'object'
            THEN
              v_ticket_train_no := NULLIF(BTRIM(COALESCE(v_ticket->'reservation_snapshot'->>'train_no', '')), '');
            END IF;

            IF jsonb_typeof(v_ticket->'tickets') = 'array' THEN
              SELECT CASE WHEN COUNT(*) = 0 THEN NULL ELSE to_jsonb(array_agg(seat_label ORDER BY seat_label)) END
              INTO v_ticket_seats
              FROM (
                SELECT DISTINCT
                  CASE
                    WHEN NULLIF(BTRIM(COALESCE(ticket_row->>'seat_no', '')), '') IS NULL THEN NULL
                    WHEN NULLIF(BTRIM(COALESCE(ticket_row->>'car_no', '')), '') IS NULL THEN BTRIM(COALESCE(ticket_row->>'seat_no', ''))
                    ELSE BTRIM(COALESCE(ticket_row->>'car_no', '')) || '-' || BTRIM(COALESCE(ticket_row->>'seat_no', ''))
                  END AS seat_label
                FROM jsonb_array_elements(v_ticket->'tickets') AS ticket_rows(ticket_row)
              ) AS normalized_seats
              WHERE seat_label IS NOT NULL;

              SELECT
                CASE
                  WHEN COUNT(*) = 0 THEN NULL
                  ELSE to_jsonb(
                    array_agg(
                      seat_class
                      ORDER BY
                        CASE seat_class WHEN 'general' THEN 0 WHEN 'special' THEN 1 ELSE 99 END,
                        seat_class
                    )
                  )
                END
              INTO v_ticket_seat_classes
              FROM (
                SELECT DISTINCT seat_class
                FROM (
                  SELECT
                    CASE
                      WHEN NULLIF(BTRIM(COALESCE(ticket_row->>'seat_class_code', '')), '') = '1' THEN 'general'
                      WHEN NULLIF(BTRIM(COALESCE(ticket_row->>'seat_class_code', '')), '') = '2' THEN 'special'
                      WHEN LOWER(COALESCE(ticket_row->>'seat_class_name', ticket_row->>'seat_class', '')) LIKE '%special%'
                        OR COALESCE(ticket_row->>'seat_class_name', ticket_row->>'seat_class', '') LIKE '%특실%'
                        THEN 'special'
                      WHEN LOWER(COALESCE(ticket_row->>'seat_class_name', ticket_row->>'seat_class', '')) LIKE '%general%'
                        OR COALESCE(ticket_row->>'seat_class_name', ticket_row->>'seat_class', '') LIKE '%일반%'
                        THEN 'general'
                      ELSE NULL
                    END AS seat_class
                  FROM jsonb_array_elements(v_ticket->'tickets') AS ticket_rows(ticket_row)
                ) AS mapped_seat_classes
                WHERE seat_class IS NOT NULL
              ) AS normalized_seat_classes;
            END IF;

            IF jsonb_typeof(v_ticket->'seat_count') = 'number' THEN
              v_ticket_seat_count := GREATEST((v_ticket->>'seat_count')::INTEGER, 0);
            ELSIF v_ticket_seats IS NOT NULL THEN
              v_ticket_seat_count := jsonb_array_length(v_ticket_seats);
            END IF;
          END IF;

          v_list_bucket := 'completed';
          IF v_task.state IN ('QUEUED', 'RUNNING', 'POLLING', 'RESERVING', 'PAYING', 'PAUSED') THEN
            v_list_bucket := 'active';
          ELSIF v_task.state = 'COMPLETED'
            AND COALESCE(v_ticket_status, '') IN ('awaiting_payment', 'reserved', 'waiting')
            AND v_ticket_paid IS DISTINCT FROM TRUE
          THEN
            v_list_bucket := 'active';
          END IF;

          v_projection_updated_at := GREATEST(v_task.updated_at, NOW());

          INSERT INTO task_realtime_events (
            task_id,
            user_id,
            module,
            state,
            spec_date,
            updated_at,
            ticket_status,
            ticket_paid,
            ticket_payment_deadline_at,
            ticket_reservation_id,
            ticket_train_no,
            ticket_seat_count,
            ticket_seats,
            ticket_seat_classes,
            list_bucket
          )
          VALUES (
            v_task.id,
            v_task.user_id,
            v_task.module,
            v_task.state,
            v_spec_date,
            v_projection_updated_at,
            v_ticket_status,
            v_ticket_paid,
            v_ticket_payment_deadline_at,
            v_ticket_reservation_id,
            v_ticket_train_no,
            v_ticket_seat_count,
            v_ticket_seats,
            v_ticket_seat_classes,
            v_list_bucket
          )
          ON CONFLICT (task_id) DO UPDATE
          SET
            user_id = EXCLUDED.user_id,
            module = EXCLUDED.module,
            state = EXCLUDED.state,
            spec_date = EXCLUDED.spec_date,
            updated_at = EXCLUDED.updated_at,
            ticket_status = EXCLUDED.ticket_status,
            ticket_paid = EXCLUDED.ticket_paid,
            ticket_payment_deadline_at = EXCLUDED.ticket_payment_deadline_at,
            ticket_reservation_id = EXCLUDED.ticket_reservation_id,
            ticket_train_no = EXCLUDED.ticket_train_no,
            ticket_seat_count = EXCLUDED.ticket_seat_count,
            ticket_seats = EXCLUDED.ticket_seats,
            ticket_seat_classes = EXCLUDED.ticket_seat_classes,
            list_bucket = EXCLUDED.list_bucket
          WHERE
            task_realtime_events.user_id IS DISTINCT FROM EXCLUDED.user_id
            OR task_realtime_events.module IS DISTINCT FROM EXCLUDED.module
            OR task_realtime_events.state IS DISTINCT FROM EXCLUDED.state
            OR task_realtime_events.spec_date IS DISTINCT FROM EXCLUDED.spec_date
            OR task_realtime_events.ticket_status IS DISTINCT FROM EXCLUDED.ticket_status
            OR task_realtime_events.ticket_paid IS DISTINCT FROM EXCLUDED.ticket_paid
            OR task_realtime_events.ticket_payment_deadline_at IS DISTINCT FROM EXCLUDED.ticket_payment_deadline_at
            OR task_realtime_events.ticket_reservation_id IS DISTINCT FROM EXCLUDED.ticket_reservation_id
            OR task_realtime_events.ticket_train_no IS DISTINCT FROM EXCLUDED.ticket_train_no
            OR task_realtime_events.ticket_seat_count IS DISTINCT FROM EXCLUDED.ticket_seat_count
            OR task_realtime_events.ticket_seats IS DISTINCT FROM EXCLUDED.ticket_seats
            OR task_realtime_events.ticket_seat_classes IS DISTINCT FROM EXCLUDED.ticket_seat_classes
            OR task_realtime_events.list_bucket IS DISTINCT FROM EXCLUDED.list_bucket;
        END
        $$;
        """
    )

    op.execute(
        """
        CREATE OR REPLACE FUNCTION sync_task_realtime_events()
        RETURNS trigger
        LANGUAGE plpgsql
        AS $$
        BEGIN
          IF TG_OP = 'DELETE' THEN
            DELETE FROM task_realtime_events WHERE task_id = OLD.id;
            RETURN OLD;
          END IF;

          PERFORM upsert_task_realtime_event_for_task(NEW.id);
          RETURN NEW;
        END
        $$;
        """
    )

    op.execute(
        """
        CREATE OR REPLACE FUNCTION sync_task_realtime_events_from_artifact()
        RETURNS trigger
        LANGUAGE plpgsql
        AS $$
        DECLARE
          v_new_task_id UUID;
          v_old_task_id UUID;
          v_new_ticket BOOLEAN;
          v_old_ticket BOOLEAN;
        BEGIN
          v_new_task_id := CASE WHEN TG_OP = 'DELETE' THEN NULL ELSE NEW.task_id END;
          v_old_task_id := CASE WHEN TG_OP = 'INSERT' THEN NULL ELSE OLD.task_id END;
          IF TG_OP = 'DELETE' THEN
            v_new_ticket := FALSE;
          ELSE
            v_new_ticket := NEW.module = 'train' AND NEW.kind = 'ticket';
          END IF;
          IF TG_OP = 'INSERT' THEN
            v_old_ticket := FALSE;
          ELSE
            v_old_ticket := OLD.module = 'train' AND OLD.kind = 'ticket';
          END IF;

          IF v_new_ticket AND v_new_task_id IS NOT NULL THEN
            PERFORM upsert_task_realtime_event_for_task(v_new_task_id);
          END IF;

          IF v_old_ticket AND v_old_task_id IS NOT NULL THEN
            IF (NOT v_new_ticket) OR v_old_task_id IS DISTINCT FROM v_new_task_id THEN
              PERFORM upsert_task_realtime_event_for_task(v_old_task_id);
            END IF;
          END IF;

          IF TG_OP = 'DELETE' THEN
            RETURN OLD;
          END IF;
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

    op.execute("DROP TRIGGER IF EXISTS trg_sync_task_realtime_events_from_artifact ON artifacts")
    op.execute(
        """
        CREATE TRIGGER trg_sync_task_realtime_events_from_artifact
        AFTER INSERT OR UPDATE OF task_id, module, kind, data_json_safe, created_at OR DELETE ON artifacts
        FOR EACH ROW
        EXECUTE FUNCTION sync_task_realtime_events_from_artifact()
        """
    )

    op.execute(
        """
        DO $$
        DECLARE
          v_task_id UUID;
        BEGIN
          FOR v_task_id IN
            SELECT id FROM tasks
          LOOP
            PERFORM upsert_task_realtime_event_for_task(v_task_id);
          END LOOP;
        END
        $$;
        """
    )



def downgrade() -> None:
    op.execute("DROP TRIGGER IF EXISTS trg_sync_task_realtime_events_from_artifact ON artifacts")
    op.execute("DROP FUNCTION IF EXISTS sync_task_realtime_events_from_artifact")
    op.execute("DROP FUNCTION IF EXISTS upsert_task_realtime_event_for_task")

    op.execute("DROP TRIGGER IF EXISTS trg_sync_task_realtime_events_delete ON tasks")
    op.execute("DROP TRIGGER IF EXISTS trg_sync_task_realtime_events_upsert ON tasks")

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

    op.execute(
        """
        ALTER TABLE task_realtime_events
        DROP CONSTRAINT IF EXISTS ck_task_realtime_events_list_bucket
        """
    )

    op.execute(
        """
        ALTER TABLE task_realtime_events
        DROP COLUMN IF EXISTS ticket_status,
        DROP COLUMN IF EXISTS ticket_paid,
        DROP COLUMN IF EXISTS ticket_payment_deadline_at,
        DROP COLUMN IF EXISTS ticket_reservation_id,
        DROP COLUMN IF EXISTS ticket_train_no,
        DROP COLUMN IF EXISTS ticket_seat_count,
        DROP COLUMN IF EXISTS ticket_seats,
        DROP COLUMN IF EXISTS ticket_seat_classes,
        DROP COLUMN IF EXISTS list_bucket
        """
    )
