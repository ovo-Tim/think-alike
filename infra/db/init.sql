create extension if not exists vector;

create table if not exists users (
    github_id bigint primary key,
    login text not null unique,
    avatar_url text,
    updated_at timestamptz not null default now()
);

create table if not exists thoughts (
    id uuid primary key,
    user_id bigint not null references users(github_id) on delete cascade,
    title varchar(120) not null,
    description text not null,
    embedding vector not null,
    embedding_dimensions integer,
    created_at timestamptz not null default now()
);

alter table thoughts alter column embedding type vector using embedding::vector;
alter table thoughts add column if not exists embedding_dimensions integer;
update thoughts
set embedding_dimensions = vector_dims(embedding)
where embedding_dimensions is null;
alter table thoughts alter column embedding_dimensions set not null;

create index if not exists thoughts_user_created_at_idx on thoughts (user_id, created_at desc);
create index if not exists thoughts_created_at_idx on thoughts (created_at desc);
create index if not exists thoughts_embedding_dimensions_idx on thoughts (embedding_dimensions);
