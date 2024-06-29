WITH before_update AS (
    SELECT balance
    FROM "user"
    WHERE id = $1
),
update_user AS (
    UPDATE "user"
    SET balance = CASE
                    WHEN balance >= $2 THEN balance + $3
                    ELSE balance
                END
    WHERE id = $1
    RETURNING balance
)
SELECT 
    (before_update.balance <> update_user.balance) AS transaction_successful
FROM 
    before_update, update_user