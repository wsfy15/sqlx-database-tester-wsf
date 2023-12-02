-- Add migration script here
CREATE TABLE todos (
  id SERIAL PRIMARY KEY,
  title varchar(255) NOT NULL,
  completed BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);