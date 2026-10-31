CREATE TABLE ai_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    ai_enabled INTEGER NOT NULL DEFAULT 0 CHECK (ai_enabled IN (0, 1)),
    provider TEXT NOT NULL DEFAULT 'openai',
    base_url TEXT NOT NULL DEFAULT '',
    model TEXT NOT NULL DEFAULT '',
    api_key TEXT
);

INSERT INTO ai_settings (id) VALUES (1);
