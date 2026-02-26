-- Linkhash registry/auth indexes + uniqueness

CREATE UNIQUE INDEX IF NOT EXISTS "idx_packages_org_name"
  ON "packages"("org_id", lower("name"));

CREATE UNIQUE INDEX IF NOT EXISTS "idx_package_versions_unique"
  ON "package_versions"("package_id", "version");

CREATE UNIQUE INDEX IF NOT EXISTS "idx_package_versions_canonical_id"
  ON "package_versions"("canonical_id")
  WHERE "canonical_id" IS NOT NULL;

CREATE INDEX IF NOT EXISTS "lh_org_members_org_idx" ON "lh_org_members"("org_id");
CREATE INDEX IF NOT EXISTS "lh_org_members_user_idx" ON "lh_org_members"("user_id");

CREATE INDEX IF NOT EXISTS "lh_rate_limit_bucket_subject_idx"
  ON "lh_rate_limit_hits"("bucket", "subject_key", "created_at");

CREATE INDEX IF NOT EXISTS "lh_audit_log_created_idx" ON "lh_audit_logs"("created_at");
CREATE INDEX IF NOT EXISTS "lh_event_log_created_idx" ON "lh_event_logs"("created_at");

CREATE INDEX IF NOT EXISTS "idx_sessions_user_id" ON "lh_sessions"("user_id");
CREATE INDEX IF NOT EXISTS "idx_sessions_token_hash" ON "lh_sessions"("session_token_hash");
CREATE INDEX IF NOT EXISTS "idx_api_tokens_user_id" ON "lh_api_tokens"("user_id");
CREATE INDEX IF NOT EXISTS "idx_api_tokens_prefix" ON "lh_api_tokens"("token_prefix");
