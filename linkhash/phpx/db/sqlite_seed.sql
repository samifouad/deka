PRAGMA foreign_keys = ON;

DROP TABLE IF EXISTS packages;

CREATE TABLE packages (
  id INTEGER PRIMARY KEY,
  org_id INTEGER,
  name TEXT NOT NULL,
  description TEXT,
  latest_version TEXT,
  download_count INTEGER DEFAULT 0,
  created_at TEXT,
  updated_at TEXT
);

INSERT INTO packages (id, org_id, name, description, latest_version, download_count, created_at, updated_at) VALUES
(1, 1, 'component', 'PHPX component primitives and JSX runtime', '0.3.0', 12450, datetime('now'), datetime('now')),
(2, 1, 'db', 'Low-level database bridge and shared primitives', '0.2.1', 9170, datetime('now'), datetime('now')),
(3, 2, 'db-sqlite', 'SQLite driver for PHPX db module', '0.1.0', 6122, datetime('now'), datetime('now')),
(4, 2, 'encoding-json', 'JSON codec under encoding/json', '0.4.2', 14220, datetime('now'), datetime('now')),
(5, 3, 'router', 'Minimal router helpers for server apps', '0.1.4', 4880, datetime('now'), datetime('now')),
(6, 3, 'auth', 'Session and token helpers for registry apps', '0.2.0', 3050, datetime('now'), datetime('now'));
