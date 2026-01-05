CREATE TABLE school_class_status_configs (
    id SERIAL PRIMARY KEY,
    school_id INT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
    status INT NOT NULL,
    label VARCHAR(32) NOT NULL,
    type VARCHAR(16) NOT NULL DEFAULT 'default',
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT school_class_status_configs_school_status_unique UNIQUE (school_id, status),
    CONSTRAINT school_class_status_configs_status_check CHECK (status IN (0, 1, 2)),
    CONSTRAINT school_class_status_configs_type_check CHECK (type IN ('default', 'success', 'warning', 'danger', 'primary'))
);
