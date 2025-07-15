-- Add migration script here

CREATE TYPE lottery_game_status AS ENUM (
    'ongoing',
    'done',
    'cancelled'
);

CREATE TYPE lottery_transaction_status AS ENUM(
    'pending',
    'done',
    'failed'
);

CREATE TABLE lottery_transactions (
    id VARCHAR  NOT NULL,
    user_name VARCHAR  NOT NULL ,
    status lottery_transaction_status NOT NULL DEFAULT 'pending',
    amount INTEGER DEFAULT 0,
    number_of_tickets INTEGER DEFAULT 0,
    transaction_id TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()

);

CREATE TABLE lottery_tickets(
    user_name VARCHAR NOT NULL ,
    number INTEGER DEFAULT 0,
    UNIQUE (user_name, number)
);

