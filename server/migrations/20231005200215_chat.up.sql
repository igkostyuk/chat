-- Add down migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    user_id         UUID        DEFAULT uuid_generate_v4()  PRIMARY KEY,
    username        VARCHAR(255) NOT NULL CHECK ( username <> '' ),
    email           VARCHAR(64)  UNIQUE NOT NULL CHECK ( email <> '' ), 
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    hashed_password TEXT NOT NULL,
    code            TEXT NOT NULL CHECK ( code <> '' )
);

CREATE TABLE IF NOT EXISTS rooms (
    room_id         UUID        DEFAULT uuid_generate_v4()  PRIMARY KEY,
    room_name        VARCHAR(255) NOT NULL CHECK ( room_name <> '' ),
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    code            TEXT NOT NULL CHECK ( code <> '' )
);

CREATE TABLE IF NOT EXISTS messages (
    message_id      UUID DEFAULT uuid_generate_v4()  PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users ON DELETE CASCADE ,
    room_id         UUID NOT NULL REFERENCES rooms ON DELETE CASCADE ,
    content         VARCHAR(255) NOT NULL CHECK ( content <> '' ),
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS members (
    user_id      UUID REFERENCES users ON DELETE CASCADE ,
    room_id      UUID REFERENCES rooms ON DELETE CASCADE ,
    code         TEXT NOT NULL CHECK ( code <> '' ),
    join_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, room_id)
);
