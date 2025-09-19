create table agents_old(
    created_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    updated_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    id blob primary key,
    provider text not null check (provider in ('google')),
    model text not null,
    constraint uq_agents_provider_model unique (provider, model)
) without rowid;

insert into agents(created_at, updated_at, id, provider, model)
    select created_at, updated_at, id, provider, model from agents where provider != 'groq';
drop table agents;

alter table agents_old rename to agents;

