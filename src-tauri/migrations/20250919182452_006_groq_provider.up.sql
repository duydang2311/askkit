create table agents_new(
    created_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    updated_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    id text primary key,
    provider text not null check (provider in ('google', 'groq')),
    model text not null,
    constraint uq_agents_provider_model unique (provider, model)
);

create index idx_agents_created_at on agents_new(created_at);

insert into agents_new(created_at, updated_at, id, provider, model)
    select created_at, updated_at, id, provider, model from agents;
drop table agents;

alter table agents_new rename to agents;

insert into agents (id, provider, model) values (X'1aac2490ddd74c49bc2534abcb00a2ec', 'groq', 'llama-3.1-8b-instant');
insert into agents (id, provider, model) values (X'778ddffff6a54596be27cbe9b8ba292f', 'groq', 'llama-3.3-70b-versatile');
insert into agents (id, provider, model) values (X'cd92d5e1ab38486c8ebb27ba211cecce', 'groq', 'meta-llama/llama-guard-4-12b');
insert into agents (id, provider, model) values (X'15684bf2cede4aaf804b8410ff63483b', 'groq', 'openai/gpt-oss-120b');
insert into agents (id, provider, model) values (X'025f9a482d4b4a9da42989872822802f', 'groq', 'openai/gpt-oss-20b');
insert into agents (id, provider, model) values (X'abfc9a71ee944b5d9ce68c8823deb9a4', 'groq', 'whisper-large-v3');
insert into agents (id, provider, model) values (X'ef9814dacf1c40538086a4babe3546e5', 'groq', 'whisper-large-v3-turbo');