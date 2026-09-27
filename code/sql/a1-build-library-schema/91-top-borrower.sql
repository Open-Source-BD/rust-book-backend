-- The member who has borrowed the most books, ever.
SELECT members.name AS member, count(loans.id) AS times_borrowed
FROM members
JOIN loans ON loans.member_id = members.id
GROUP BY members.id, members.name
ORDER BY times_borrowed DESC, members.name
LIMIT 1;
