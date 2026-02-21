# Manual Regression: `deka init` + `deka task dev` should serve default page

## Setup
1. Create a new directory (empty).
2. Run `deka init`.
3. Run `deka task dev`.
4. Visit `http://localhost:8530`.

## Expected
- Returns HTTP 200.
- Page contains “Project initialized. Edit app/main.phpx”.

## Current (bug)
- Returns HTTP 404 “Not Found”.

## Notes
- Root cause: runtime PHP router expects `app/page.phpx` or `app/index.phpx`.
- `deka init` writes `app/main.phpx`, so a fresh init yields no matching page.
