# linkha.sh PHPX Registry - Project Tasks

## MVP Features (Priority Order)

### ✅ Phase 1: Basic Foundation - COMPLETED!
- [x] Set up project structure with proper php_modules
- [x] Get basic PHPX execution working (test_basic.phpx)
- [x] Copy working deka.php bridge and core modules
- [x] Create minimal HTTP server using serve() API
- [x] Implement basic routing (/, /api/packages, /api/search)
- [x] Add Tailwind CSS styling to homepage
- [x] Test server startup and basic endpoints
- [x] Working PHPX registry server at http://localhost:8530

### Phase 2: Core Registry Functionality  
- [x] Create PostgreSQL database schema
- [x] Set up database connection layer
- [x] Implement package search functionality
- [x] Create package listing API endpoints
- [x] Add package detail pages (SSR + API)
- [x] Test API endpoints with curl/CLI

### Phase 3: Authentication & Publishing
- [x] Implement Bluesky OAuth flow (login + callback exchange + profile sync)
- [x] Create organization/user management (dev baseline)
- [x] Add package publishing endpoint (API baseline with org-role enforcement)
- [x] Add org membership APIs (owner/publisher/maintainer)
- [x] Add package visibility controls (`public`/`private`)
- [x] Add route-level auth rate limits (login/callback, PAT mutate, publish mutate)
- [x] Implement package upload/download (local DB-backed artifact bytes for MVP)
- [x] Integrate Cloudflare R2 storage (optional primary with automatic Postgres inline fallback)
- [x] Test publish workflow end-to-end (local publish/install/artifact round-trip)
- [x] Implement session auth + PAT scopes (`read`, `read:write`, `read:write:delete`)

### Phase 4: Polish & UX
- [x] Enhance homepage with featured packages
- [x] Add organization profile pages (SSR + API)
- [x] Implement package statistics (API + homepage cards)
- [x] Create playground integration
- [x] Add error handling and logging
- [x] Docker deployment setup

## Runtime Issues Found (For Future Runtime Development)

### Remaining Issue: Cloud Artifact Backend
- **Problem**: Artifact bytes are currently stored inline in Postgres for MVP.
- **Current Status**: Local workflow works, but R2 backend is not integrated yet.
- **Impact**: No production object storage path yet.

## Working Patterns Discovered

### ✅ Basic PHPX Execution
```phpx
function greet($name: string): string {
    return "Hello, " . $name;
}
echo greet("World");
```

### ✅ Module System
```phpx
import { Result, Ok, Err } from 'core/result';
```

### ✅ File Structure
- `deka.lock` required in project root
- `php_modules/deka.php` bridge file required
- `php_modules/core/` for core modules
- `php_modules/.cache/` for compiled files

### ✅ HTTP Server with deka serve
```bash
deka serve main.phpx  # Starts server on port 8530
```

### ✅ PHPX Web Application Patterns
- Use `$_SERVER['REQUEST_URI']` for routing
- `header()` function for HTTP headers
- `json_encode()` for JSON responses
- Standard PHP functions work in PHPX
