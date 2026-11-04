CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE screens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    width INTEGER NOT NULL DEFAULT 390 CHECK (width BETWEEN 100 AND 4000),
    height INTEGER NOT NULL DEFAULT 844 CHECK (height BETWEEN 100 AND 4000),
    frame TEXT NOT NULL DEFAULT 'phone' CHECK (frame IN ('phone', 'tablet', 'browser', 'none')),
    document_version INTEGER NOT NULL DEFAULT 1,
    document JSONB NOT NULL DEFAULT '{"version":1,"elements":[]}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX screens_project_order ON screens(project_id, position);
