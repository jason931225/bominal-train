alter table if exists train_search_sessions
    add column if not exists passengers_json jsonb;

update train_search_sessions
set passengers_json = jsonb_build_array(
    jsonb_build_object(
        'kind',
        'adult',
        'count',
        greatest(passenger_count, 1)
    )
)
where passengers_json is null;

update train_search_sessions
set passengers_json = '[{"kind":"adult","count":1}]'::jsonb
where passengers_json is null
   or jsonb_typeof(passengers_json) <> 'array'
   or jsonb_array_length(passengers_json) = 0;

alter table if exists train_search_sessions
    alter column passengers_json set default '[{"kind":"adult","count":1}]'::jsonb;

alter table if exists train_search_sessions
    alter column passengers_json set not null;

do $$
begin
    if not exists (
        select 1
        from pg_constraint
        where conname = 'chk_train_search_sessions_passengers_json_array'
    ) then
        alter table train_search_sessions
            add constraint chk_train_search_sessions_passengers_json_array
                check (
                    jsonb_typeof(passengers_json) = 'array'
                    and jsonb_array_length(passengers_json) > 0
                );
    end if;
end
$$;
