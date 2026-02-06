# PHPX Stdlib Migration Guide (Current)

## Module path changes
- Canonical JSON path: `encoding/json`
- Canonical DB driver paths:
  - `db/postgres`
  - `db/mysql`
  - `db/sqlite`

## Compatibility paths (temporary)
- `json` proxies to `encoding/json`
- `postgres` proxies to `db/postgres`
- `mysql` proxies to `db/mysql`
- `sqlite` proxies to `db/sqlite`

## Recommended imports for new code
```php
import { json_encode } from 'encoding/json'
import { connect, query_one } from 'db/postgres'
```

## Existing code migration
1. Replace `from 'json'` with `from 'encoding/json'`
2. Replace `from 'postgres'` with `from 'db/postgres'`
3. Replace `from 'mysql'` with `from 'db/mysql'`
4. Replace `from 'sqlite'` with `from 'db/sqlite'`

## Contract guarantees
- `db/open_handle` returns `Result<int, string>`
- `db/query` returns `Result<{ rows: array<Object> }, string>`
- `db/query_one` returns `Result<Object, string>`
- `db/exec` returns `Result<{ affected_rows: int }, string>`

## Deprecation window
- Compatibility paths remain through MVP.
- Removal target: first post-MVP breaking release.
