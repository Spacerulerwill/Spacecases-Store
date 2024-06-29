CREATE TABLE "user" (
    id BIGINT NOT NULL PRIMARY KEY,
    balance BIGINT NOT NULL,
    last_claim DATE NOT NULL,
    claim_streak BIGINT NOT NUll
);