SELECT 12.345::numeric(10,2) AS rounds_up,
       12.344::numeric(10,2) AS rounds_down,
       12.5::numeric(10,2)   AS pads_with_zero;
SELECT 123456789.5::numeric(10,2) AS too_many_digits;
