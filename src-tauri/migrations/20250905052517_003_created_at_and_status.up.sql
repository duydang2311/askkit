create table chats_new(
    created_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    id text primary key not null,
    title text not null
) without rowid;

create table chat_messages_new (
    created_at integer not null default (cast(unixepoch('now', 'subsecond') * 1000 as integer)),
    chat_id text not null,
    id text primary key not null,
    role text not null,
    content text not null,
    status text not null default 'completed' check (status in ('pending', 'completed', 'failed')),
    constraint fk_chat_messages_chats_chat_id foreign key (chat_id) references chats(id) on delete cascade
) without rowid;

insert into chats_new(created_at, id, title) select created_at, id, title from chats;
insert into chat_messages_new(created_at, chat_id, id, role, content, status)
    select created_at, chat_id, id, role, content, status from chat_messages;

drop table chats;
drop table chat_messages;

alter table chats_new rename to chats;
alter table chat_messages_new rename to chat_messages;
