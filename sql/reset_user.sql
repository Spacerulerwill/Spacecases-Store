UPDATE "user"
SET balance = 0,
last_claim = '0001-01-01',
claim_streak = 0
WHERE id = $1