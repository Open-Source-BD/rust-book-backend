SELECT 7 / 2 AS whole_numbers, 7 / 2.0 AS with_decimals;
SELECT 0.1::double precision + 0.2::double precision AS floating_point,
       0.1::numeric + 0.2::numeric AS exact_numeric;
SELECT 2147483647 + 1 AS too_big_for_integer;
SELECT 2147483647::bigint + 1 AS fits_in_bigint;
