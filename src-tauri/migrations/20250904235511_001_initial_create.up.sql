create table chats (
    created_at integer not null default (unixepoch('now', 'subsecond')),
    id blob primary key not null,
    title text not null
);

create table chat_messages (
    created_at integer not null default (unixepoch('now', 'subsecond')),
    chat_id blob not null,
    id blob primary key not null,
    role text not null,
    content text not null,
    constraint fk_chat_messages_chats_chat_id foreign key (chat_id) references chats(id) on delete cascade
);