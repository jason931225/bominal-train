create index if not exists ix_runtime_job_events_job_id_id
    on runtime_job_events (job_id, id);
