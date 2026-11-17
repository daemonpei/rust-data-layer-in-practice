-- Tenants are the isolation boundary for every other table.
CREATE TABLE IF NOT EXISTS tenants (
    tenant_id   UUID PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    plan        TEXT NOT NULL DEFAULT 'starter',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS tenants_slug_idx ON tenants (slug);
