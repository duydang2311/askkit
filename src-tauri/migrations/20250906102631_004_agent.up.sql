create table agents (
    created_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    updated_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    id text primary key,
    provider text not null check (provider in ('gemini')),
    model text not null,
    constraint uq_agents_provider_model unique (provider, model)
);

create table agent_configs (
    created_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    updated_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    agent_id text primary key,
    api_key text null,
    constraint fk_agent_configs_agents_agent_id foreign key (agent_id) references agents(id) on delete cascade
);

create table agent_providers (
    created_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    updated_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    id text not null primary key,
    provider text not null check (provider in ('gemini')),
    api_key text null
);

create table current_agent (
    created_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    updated_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    id integer not null default 1 primary key check(id = 1),
    agent_id text not null,
    constraint fk_current_agent_agents_agent_id foreign key (agent_id) references agents(id) on delete cascade
);

create trigger tr_agents_set_updated_at
after update on agents
for each row
when new.updated_at = old.updated_at
begin
    update agents
    set updated_at = (cast(unixepoch('now', 'subsecond') * 1000 as integer))
    where rowid = new.rowid;
end;

create trigger tr_agent_configs_set_updated_at
after update on agent_configs
for each row
when new.updated_at = old.updated_at
begin
    update agent_configs
    set updated_at = (cast(unixepoch('now', 'subsecond') * 1000 as integer))
    where rowid = new.rowid;
end;

create trigger tr_agent_providers_set_updated_at
after update on agent_providers
for each row
when new.updated_at = old.updated_at
begin
    update agent_providers
    set updated_at = (cast(unixepoch('now', 'subsecond') * 1000 as integer))
    where rowid = new.rowid;
end;

create trigger tr_current_agent_set_updated_at
after update on current_agent
for each row
when new.updated_at = old.updated_at
begin
    update current_agent
    set updated_at = (cast(unixepoch('now', 'subsecond') * 1000 as integer))
    where rowid = new.rowid;
end;
