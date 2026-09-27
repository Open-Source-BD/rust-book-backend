SELECT pg_size_pretty(pg_relation_size('orders'))                 AS table_size,
       pg_size_pretty(pg_relation_size('orders_pkey'))            AS pkey_index,
       pg_size_pretty(pg_relation_size('orders_customer_id_idx')) AS customer_id_index,
       pg_size_pretty(pg_relation_size('orders_status_idx'))      AS status_index;
