CREATE COLLATION IF NOT EXISTS ignoreCase (
  provider = 'icu',
  locale = 'und-u-ks-level2',
  deterministic = false
);
-- Add up migration script here
CREATE TABLE IF NOT EXISTS roles(
    id serial PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS role_permissions(
    id serial PRIMARY KEY,
    role_id integer NOT NULL,
        -- Relates to roles table
        CONSTRAINT FK_role_permissions_role_id
            FOREIGN KEY (role_id)
            REFERENCES roles(id)
            ON DELETE CASCADE,
    permission VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO roles(name, description)
    VALUES ('Admin', 'Admin role');

INSERT INTO roles(name, description)
    VALUES ('Clinician', 'Clinician role');

INSERT INTO role_permissions(role_id, permission)
    VALUES
    ((SELECT id FROM roles WHERE name = 'Admin'), 'Admin');

INSERT INTO role_permissions(role_id, permission)
    VALUES
    ((SELECT id FROM roles WHERE name = 'Clinician'), 'participants:read'),
    ((SELECT id FROM roles WHERE name = 'Clinician'), 'participants:update'),
    ((SELECT id FROM roles WHERE name = 'Clinician'), 'schedule:read'),
    ((SELECT id FROM roles WHERE name = 'Clinician'), 'schedule:manage');

CREATE TABLE IF NOT EXISTS users(
    id serial PRIMARY KEY,
    username VARCHAR(255) NOT NULL COLLATE ignoreCase,
    email VARCHAR(320) NOT NULL COLLATE ignoreCase,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO users(username, email, first_name, last_name)
    VALUES ('admin','admin@example.com', 'Admin', 'User');

CREATE TABLE IF NOT EXISTS user_permissions(
    id serial PRIMARY KEY,
    user_id integer NOT NULL,
        -- Relates to users table
        CONSTRAINT FK_user_permissions_user_id
            FOREIGN KEY (user_id)
            REFERENCES users(id)
            ON DELETE CASCADE,
    permission VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_roles(
    id serial PRIMARY KEY,
    user_id integer NOT NULL,
    role_id integer NOT NULL,
    -- Relates to users table
        CONSTRAINT FK_user_roles_user_id
            FOREIGN KEY (user_id)
            REFERENCES users(id)
            ON DELETE CASCADE,
    -- Relates to roles table
        CONSTRAINT FK_user_roles_role_id
            FOREIGN KEY (role_id)
            REFERENCES roles(id)
            ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO user_roles(user_id, role_id)
    VALUES
    ((SELECT id FROM users WHERE username = 'admin'), (SELECT id FROM roles WHERE name = 'Admin'));

CREATE TABLE IF NOT EXISTS user_authentication_password(
    id serial PRIMARY KEY,
    user_id integer NOT NULL,
    -- Relates to users table
        CONSTRAINT FK_user_authentication_password_user_id
            FOREIGN KEY (user_id)
            REFERENCES users(id)
            ON DELETE CASCADE,
    password TEXT,
    requires_reset BOOLEAN DEFAULT FALSE,
    updated_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO user_authentication_password(user_id, password)
    VALUES
    ((SELECT id FROM users WHERE username = 'admin'), '$argon2i$v=19$m=16,t=2,p=1$VjJ1RHZic2l4VXFxbUNaMA$ewDhK5UqOdofv+BhAs+FUg');


CREATE TABLE IF NOT EXISTS user_login_attempts(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id integer,
    -- Relates to users table
        CONSTRAINT FK_user_login_attempts_user_id
            FOREIGN KEY (user_id)
            REFERENCES users(id)
            ON DELETE CASCADE,
    ip_address VARCHAR(255),
    -- HTTP Headers such as User-Agent
    additional_footprint JSONB,
    success BOOLEAN NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE  DEFAULT CURRENT_TIMESTAMP
);
