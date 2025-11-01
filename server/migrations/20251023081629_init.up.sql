-- schools table
CREATE TABLE schools (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add up migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    wechat_openid VARCHAR(255) UNIQUE,
    wechat_unionid VARCHAR(255) UNIQUE,
    wechat_nickname VARCHAR(255),
    phone VARCHAR(255) UNIQUE,
    wechat_avatar_url TEXT,
    school_id INT REFERENCES schools(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
INSERT INTO "users" (  "username", "password_hash") VALUES ( 'admin', '$2b$12$/MZyRsK.DcYHh6x4qCy6IOjxO/Wd4RlPSbW.7OiAYqTY4U4CipDIS');
CREATE INDEX idx_users_school_id ON users (school_id);


CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO "roles" ( "name", "description") VALUES ( 'admin', '管理员');
INSERT INTO "roles" ( "name", "description") VALUES ( 'user', '用户');
INSERT INTO "roles" ( "name", "description") VALUES ( 'teacher', '教师');

-- Permissions Table
CREATE TABLE permissions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    CONSTRAINT "action_check" CHECK (action IN ('READ', 'CREATE', 'UPDATE', 'DELETE','*'))
);

INSERT INTO "permissions" ( "name", "resource", "action", "description") VALUES ( 'all', '*', '*', '所有资源');
-- User_Roles Join Table
CREATE TABLE user_roles (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

INSERT INTO "user_roles" ( "user_id", "role_id") VALUES ( 1, 1);

-- Role_Permissions Join Table
CREATE TABLE role_permissions (
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

INSERT INTO "role_permissions" ( "role_id", "permission_id") VALUES ( 1, 1);

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



