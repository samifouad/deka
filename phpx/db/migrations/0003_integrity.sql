-- Add package integrity hashes for registry verification
ALTER TABLE "package_versions" ADD COLUMN IF NOT EXISTS "integrity_algo" TEXT;
ALTER TABLE "package_versions" ADD COLUMN IF NOT EXISTS "module_graph_hash" TEXT;
ALTER TABLE "package_versions" ADD COLUMN IF NOT EXISTS "fs_graph_hash" TEXT;

CREATE INDEX IF NOT EXISTS "idx_package_versions_module_graph_hash" ON "package_versions"("module_graph_hash");
CREATE INDEX IF NOT EXISTS "idx_package_versions_fs_graph_hash" ON "package_versions"("fs_graph_hash");
