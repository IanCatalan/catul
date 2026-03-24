use tauri_plugin_sql::{Migration, MigrationKind};

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "initial_schema",
            kind: MigrationKind::Up,
            sql: "
                PRAGMA journal_mode = WAL;
                PRAGMA foreign_keys = ON;

                -- projects
                CREATE TABLE IF NOT EXISTS projects (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    name        TEXT    NOT NULL,
                    source_lang TEXT    NOT NULL,
                    target_lang TEXT    NOT NULL,
                    status      TEXT    NOT NULL DEFAULT 'active'
                                CHECK (status IN ('active', 'archived', 'completed')),
                    word_count  INTEGER NOT NULL DEFAULT 0,
                    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
                    updated_at  INTEGER NOT NULL DEFAULT (unixepoch())
                );

                -- documents
                CREATE TABLE IF NOT EXISTS documents (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id    INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                    filename      TEXT    NOT NULL,
                    format        TEXT    NOT NULL
                                  CHECK (format IN ('xliff','docx','html','po','srt','txt','idml')),
                    filepath      TEXT    NOT NULL,
                    segment_count INTEGER NOT NULL DEFAULT 0,
                    created_at    INTEGER NOT NULL DEFAULT (unixepoch())
                );

                -- segments
                CREATE TABLE IF NOT EXISTS segments (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    document_id   INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
                    segment_index INTEGER NOT NULL,
                    source        TEXT    NOT NULL,
                    status        TEXT    NOT NULL DEFAULT 'untranslated'
                                  CHECK (status IN ('untranslated','translated','reviewed','rejected')),
                    locked        INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0, 1)),
                    created_at    INTEGER NOT NULL DEFAULT (unixepoch()),
                    updated_at    INTEGER NOT NULL DEFAULT (unixepoch()),
                    UNIQUE (document_id, segment_index)
                );

                -- translations (full history per segment)
                CREATE TABLE IF NOT EXISTS translations (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    segment_id    INTEGER NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
                    target        TEXT    NOT NULL DEFAULT '',
                    origin        TEXT    NOT NULL DEFAULT 'manual'
                                  CHECK (origin IN ('manual','tm','mt','pretranslated')),
                    match_percent INTEGER NOT NULL DEFAULT 0 CHECK (match_percent BETWEEN 0 AND 100),
                    is_current    INTEGER NOT NULL DEFAULT 1 CHECK (is_current IN (0, 1)),
                    created_at    INTEGER NOT NULL DEFAULT (unixepoch())
                );

                -- translation memory
                CREATE TABLE IF NOT EXISTS translation_memory (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    source_lang TEXT    NOT NULL,
                    target_lang TEXT    NOT NULL,
                    source      TEXT    NOT NULL,
                    target      TEXT    NOT NULL,
                    use_count   INTEGER NOT NULL DEFAULT 1,
                    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
                    updated_at  INTEGER NOT NULL DEFAULT (unixepoch()),
                    UNIQUE (source_lang, target_lang, source)
                );

                -- FTS5 virtual table for fuzzy matching on the TM
                CREATE VIRTUAL TABLE IF NOT EXISTS tm_fts USING fts5(
                    source,
                    target,
                    content       = 'translation_memory',
                    content_rowid = 'id',
                    tokenize      = 'unicode61'
                );

                -- glossaries
                CREATE TABLE IF NOT EXISTS glossaries (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    name        TEXT    NOT NULL,
                    source_lang TEXT    NOT NULL,
                    target_lang TEXT    NOT NULL,
                    created_at  INTEGER NOT NULL DEFAULT (unixepoch())
                );

                -- terms
                CREATE TABLE IF NOT EXISTS terms (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    glossary_id INTEGER NOT NULL REFERENCES glossaries(id) ON DELETE CASCADE,
                    source_term TEXT    NOT NULL,
                    target_term TEXT    NOT NULL,
                    domain      TEXT,
                    definition  TEXT,
                    notes       TEXT,
                    forbidden   INTEGER NOT NULL DEFAULT 0 CHECK (forbidden IN (0, 1)),
                    created_at  INTEGER NOT NULL DEFAULT (unixepoch())
                );

                -- pivot table: projects <-> glossaries
                CREATE TABLE IF NOT EXISTS project_glossaries (
                    project_id  INTEGER NOT NULL REFERENCES projects(id)   ON DELETE CASCADE,
                    glossary_id INTEGER NOT NULL REFERENCES glossaries(id) ON DELETE CASCADE,
                    PRIMARY KEY (project_id, glossary_id)
                );

                -- mt_providers (DeepL, OpenAI, etc.)
                CREATE TABLE IF NOT EXISTS mt_providers (
                    id                INTEGER PRIMARY KEY AUTOINCREMENT,
                    name              TEXT    NOT NULL UNIQUE
                                      CHECK (name IN ('deepl','openai','google','modernmt')),
                    encrypted_api_key TEXT,
                    api_url           TEXT,
                    enabled           INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0, 1)),
                    is_default        INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
                    char_count_used   INTEGER NOT NULL DEFAULT 0,
                    char_limit        INTEGER,
                    updated_at        INTEGER NOT NULL DEFAULT (unixepoch())
                );

                -- qa_results
                CREATE TABLE IF NOT EXISTS qa_results (
                    id         INTEGER PRIMARY KEY AUTOINCREMENT,
                    segment_id INTEGER NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
                    rule_type  TEXT    NOT NULL
                               CHECK (rule_type IN (
                                   'missing_number','extra_number','untranslated_tag',
                                   'broken_tag','forbidden_term','missing_term',
                                   'double_space','trailing_space','punctuation_mismatch',
                                   'untranslated_source','empty_target','length_exceeded'
                               )),
                    message    TEXT    NOT NULL,
                    severity   TEXT    NOT NULL DEFAULT 'warning'
                               CHECK (severity IN ('error','warning','info')),
                    resolved   INTEGER NOT NULL DEFAULT 0 CHECK (resolved IN (0, 1)),
                    created_at INTEGER NOT NULL DEFAULT (unixepoch())
                );

                -- project_stats (cached word counts per match category)
                CREATE TABLE IF NOT EXISTS project_stats (
                    id             INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id     INTEGER NOT NULL UNIQUE REFERENCES projects(id) ON DELETE CASCADE,
                    words_new      INTEGER NOT NULL DEFAULT 0,
                    words_repeated INTEGER NOT NULL DEFAULT 0,
                    words_fuzzy_75 INTEGER NOT NULL DEFAULT 0,
                    words_fuzzy_85 INTEGER NOT NULL DEFAULT 0,
                    words_fuzzy_95 INTEGER NOT NULL DEFAULT 0,
                    words_exact    INTEGER NOT NULL DEFAULT 0,
                    words_context  INTEGER NOT NULL DEFAULT 0,
                    updated_at     INTEGER NOT NULL DEFAULT (unixepoch())
                );

                -- settings (key/value app configuration)
                CREATE TABLE IF NOT EXISTS settings (
                    id       INTEGER PRIMARY KEY AUTOINCREMENT,
                    key      TEXT NOT NULL UNIQUE,
                    value    TEXT NOT NULL,
                    category TEXT NOT NULL DEFAULT 'general'
                );

                -- indexes
                CREATE INDEX IF NOT EXISTS idx_projects_status        ON projects(status);
                CREATE INDEX IF NOT EXISTS idx_documents_project      ON documents(project_id);
                CREATE INDEX IF NOT EXISTS idx_segments_document      ON segments(document_id);
                CREATE INDEX IF NOT EXISTS idx_segments_status        ON segments(status);
                CREATE INDEX IF NOT EXISTS idx_translations_segment   ON translations(segment_id);
                CREATE INDEX IF NOT EXISTS idx_translations_current   ON translations(segment_id, is_current);
                CREATE INDEX IF NOT EXISTS idx_tm_langs               ON translation_memory(source_lang, target_lang);
                CREATE INDEX IF NOT EXISTS idx_terms_glossary         ON terms(glossary_id);
                CREATE INDEX IF NOT EXISTS idx_terms_source           ON terms(source_term COLLATE NOCASE);
                CREATE INDEX IF NOT EXISTS idx_qa_segment             ON qa_results(segment_id);
                CREATE INDEX IF NOT EXISTS idx_qa_resolved            ON qa_results(resolved);

                -- triggers: keep tm_fts in sync with translation_memory
                CREATE TRIGGER IF NOT EXISTS tm_fts_insert AFTER INSERT ON translation_memory BEGIN
                    INSERT INTO tm_fts(rowid, source, target) VALUES (new.id, new.source, new.target);
                END;

                CREATE TRIGGER IF NOT EXISTS tm_fts_update AFTER UPDATE ON translation_memory BEGIN
                    INSERT INTO tm_fts(tm_fts, rowid, source, target) VALUES ('delete', old.id, old.source, old.target);
                    INSERT INTO tm_fts(rowid, source, target) VALUES (new.id, new.source, new.target);
                END;

                CREATE TRIGGER IF NOT EXISTS tm_fts_delete AFTER DELETE ON translation_memory BEGIN
                    INSERT INTO tm_fts(tm_fts, rowid, source, target) VALUES ('delete', old.id, old.source, old.target);
                END;

                -- trigger: enforce only one active translation per segment
                CREATE TRIGGER IF NOT EXISTS trg_translations_set_current
                    BEFORE INSERT ON translations
                    WHEN NEW.is_current = 1
                BEGIN
                    UPDATE translations SET is_current = 0
                    WHERE segment_id = NEW.segment_id AND is_current = 1;
                END;

                -- trigger: auto-update updated_at on projects
                CREATE TRIGGER IF NOT EXISTS trg_projects_updated
                    AFTER UPDATE ON projects
                BEGIN
                    UPDATE projects SET updated_at = unixepoch() WHERE id = NEW.id;
                END;

                -- trigger: auto-update updated_at on segments
                CREATE TRIGGER IF NOT EXISTS trg_segments_updated
                    AFTER UPDATE ON segments
                BEGIN
                    UPDATE segments SET updated_at = unixepoch() WHERE id = NEW.id;
                END;

                -- seed data: MT providers (disabled until user adds their API key)
                INSERT OR IGNORE INTO mt_providers (name, api_url, enabled, is_default) VALUES
                    ('deepl',    'https://api-free.deepl.com/v2',      0, 0),
                    ('openai',   'https://api.openai.com/v1',          0, 0),
                    ('google',   'https://translation.googleapis.com', 0, 0),
                    ('modernmt', 'https://api.modernmt.com',           0, 0);

                -- seed data: default app settings
                INSERT OR IGNORE INTO settings (key, value, category) VALUES
                    ('ui_language',         'en',        'general'),
                    ('auto_save_to_tm',     '1',         'general'),
                    ('auto_propagate_reps', '1',         'general'),
                    ('min_fuzzy_threshold', '75',        'general'),
                    ('editor_font_size',    '16',        'editor'),
                    ('editor_font_family',  'monospace', 'editor'),
                    ('editor_spellcheck',   '1',         'editor'),
                    ('qa_check_on_confirm', '1',         'qa'),
                    ('qa_block_on_error',   '0',         'qa'),
                    ('mt_auto_suggest',     '0',         'mt'),
                    ('mt_save_to_tm',       '0',         'mt');
            ",
        },
    ]
}