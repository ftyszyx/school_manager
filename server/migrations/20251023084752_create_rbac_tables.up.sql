-- Roles Table
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO "roles" ( "id", "name", "description") VALUES ( 1, 'admin', '管理员');
INSERT INTO "roles" ( "id", "name", "description") VALUES ( 2, 'user', '用户');
INSERT INTO "roles" ( "id", "name", "description") VALUES ( 3, 'teacher', '教师');

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

INSERT INTO "permissions" ( "id", "name", "resource", "action", "description") VALUES ( 1, 'all', '*', '*', '所有资源');
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