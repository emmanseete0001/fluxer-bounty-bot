-- Add migration script here

ALTER TABLE guilds ADD COLUMN delete_commands BOOLEAN NOT NULL DEFAULT FALSE;