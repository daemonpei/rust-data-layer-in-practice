-- The hot table in Ledgerly: metered usage, one row per billable event.
-- The UNIQUE constraint is the idempotency guard described in Chapter 23.
CREATE TABLE IF NOT EXISTS usage_events (
    event_id     UUID NOT NULL,
    tenant_id    UUID NOT NULL REFERENCES tenants (tenant_id),
    metric       TEXT NOT NULL,
    amount       BIGINT NOT NULL CHECK (amount >= 0),
    occurred_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, event_id),
    UNIQUE (tenant_id, event_id)
);

CREATE INDEX IF NOT EXISTS usage_events_tenant_time_idx
    ON usage_events (tenant_id, occurred_at DESC);
