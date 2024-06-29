WITH before_update AS (
    SELECT
        last_claim,
        claim_streak
    FROM "user"
    WHERE id = $1
),
update_user AS (
    UPDATE "user"
    SET 
        balance = CASE
            WHEN CURRENT_DATE >= last_claim + INTERVAL '1 day' THEN balance + LEAST(($2 + $3 * claim_streak), $4)
            ELSE balance
        END,
        claim_streak = CASE
            WHEN CURRENT_DATE >= last_claim + INTERVAL '1 day' THEN claim_streak + 1
            ELSE claim_streak
        END,
        last_claim = CASE
            WHEN CURRENT_DATE >= last_claim + INTERVAL '1 day' THEN CURRENT_DATE
            ELSE last_claim
        END
    WHERE id = $1
    RETURNING last_claim, balance, claim_streak
)
SELECT 
    before_update.last_claim < CURRENT_DATE AS update_successful,
    update_user.balance,
    update_user.claim_streak,
    LEAST(($2 + $3 * before_update.claim_streak), $4) AS change
FROM before_update
JOIN update_user ON true;
