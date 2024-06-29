UPDATE "user"
SET balance = balance + $1
WHERE id = $2