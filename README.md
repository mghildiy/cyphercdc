## pg directory structure
/var/lib/postgresql
/var/lib/postgresql/data
databases: /var/lib/postgresql/data/base

## publication
select * from pg_publication
select * from pg_publication_rel
select * from pg_publication_tables
create publication: CREATE PUBLICATION scopes_pub FOR TABLE scopes;

## replication slot
select * from pg_replication_slots
one replication slot per consumer is common pattern
tracks LSN(log sequence number) for consumer
create slot: SELECT * FROM pg_create_logical_replication_slot('scope_slot', 'pgoutput');
The replication client tells Postgres which publications it wants to subscribe to when it starts streaming:
    START_REPLICATION SLOT scope_slot LOGICAL 0/0 (proto_version '1', publication_names 'scopes_pub');
Slot = tracks WAL and maintains state for a consumer.
Publication = defines which tables’ changes are visible to logical decoding.
Connection/stream = client chooses which publications it wants from a slot.

Enabling replication for user:
    select * from pg_user
    ALTER USER <username> REPLICATION;

Implications:
A single slot can serve multiple publications, if the client requests them.
A single publication can be consumed by multiple slots/consumers independently.

Flow:
Publication → Replication Slot → WAL Sender → Consumer

Important Settings:
wal_level = logical – must be enabled for logical replication.
max_wal_senders – maximum WAL sender processes.
max_replication_slots – must be high enough to accommodate all slots.
wal_keep_size – ensures WAL files are retained until sent.
max_slot_wal_keep_size – upper bound of WAL storage per slot.

Key points:
WAL Sender does the heavy lifting: reads WAL, converts to logical changes, streams to consumer.
If the consumer lags or stops acknowledging, WAL files accumulate.
Monitoring:
pg_stat_replication – check WAL sender and subscriber status.
pg_replication_slots – monitor lag and retained WAL size.
Performance tuning: adjust max_wal_senders, max_replication_slots, and max_slot_wal_keep_size according to system load.

Protocol implementation:
CREATE SUBSCRIPTION my_subscription
CONNECTION 'host=localhost port=5432 user=replication_user password=replication_password dbname=publisher_database'
PUBLICATION my_publication;