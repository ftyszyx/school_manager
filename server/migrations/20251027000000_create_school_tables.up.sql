-- schools table
CREATE TABLE schools (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE classes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    --年级
    grade int NOT NULL,
    --班级
    class int NOT NULL,
    school_id INT NOT NULL REFERENCES schools(id) ON DELETE CASCADE,
    --状态 0 已放学，1 上课中，2 放学中
    status int NOT NULL DEFAULT 0, -- 0: dismissed, 1: ongoing, 2: dismissing
    -- 口令
    password VARCHAR(255) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT "class_status_check" CHECK (status IN (0, 1, 2))
);

-- Create a mapping table for teachers and classes they manage
CREATE TABLE teacher_classes (
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    class_id INT NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, class_id)
);

-- Create indexes for foreign keys
CREATE INDEX idx_classes_school_id ON classes (school_id);
CREATE INDEX idx_teacher_classes_user_id ON teacher_classes (user_id);
CREATE INDEX idx_teacher_classes_class_id ON teacher_classes (class_id);
