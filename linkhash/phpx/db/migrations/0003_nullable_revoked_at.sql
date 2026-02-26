-- Allow revoked_at to be NULL for active rows
ALTER TABLE "lh_sessions" ALTER COLUMN "revoked_at" DROP NOT NULL;
ALTER TABLE "lh_api_tokens" ALTER COLUMN "revoked_at" DROP NOT NULL;
ALTER TABLE "lh_org_members" ALTER COLUMN "revoked_at" DROP NOT NULL;
