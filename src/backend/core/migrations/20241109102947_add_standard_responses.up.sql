-- Add up migration script here
CREATE TABLE IF NOT EXISTS _default_questions(
    id serial PRIMARY KEY,
    file_name VARCHAR(255) NOT NULL,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS locations(
    id serial PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    program VARCHAR(255) NOT NULL,
    parent_location INTEGER,
        CONSTRAINT FK_locations_parent_location
            FOREIGN KEY (parent_location)
            REFERENCES locations(id)
            ON DELETE CASCADE,
    red_cap_connection_rules JSONB DEFAULT '{}'
);

INSERT INTO locations(name, program, red_cap_connection_rules) VALUES
    ('Church Hill House', 'RHWP',
        '
            {
                "visit": {
                    "rhwp_location_visit": 1
                },
                "participant": {
                    "rhwp_location": 1
                }
            }
        '
    ),
    ('Dominion Place', 'RHWP',
        '
            {
                "visit": {
                    "rhwp_location_visit": 2
                },
                "participant": {
                    "rhwp_location": 2
                }
            }
        '
    ),
    ('Highland Park', 'RHWP',
        '
            {
                "visit": {
                    "rhwp_location_visit": 3
                },
                "participant": {
                    "rhwp_location": 3
                }
            }
    '),
    ('Randolph Place', 'RHWP',
        '
            {
                "visit": {
                    "rhwp_location_visit": 4
                },
                "participant": {
                    "rhwp_location": 4
                }
            }
        '
    ),
    ('4th Ave', 'RHWP',
        '
            {
                "visit": {
                    "rhwp_location_visit": 5
                },
                "participant": {
                    "rhwp_location": 5
                }
            }
    '),
    ('Health Hub', 'RHWP',
        '
            {
                "visit": {
                    "rhwp_location_visit": 6
                },
                "participant": {
                    "rhwp_location": 6
                }
            }
    '),
    ('The Rosa', 'RHWP',
        '
            {
                "visit": {
                    "rhwp_location_visit": 7
                },
                "participant": {
                    "rhwp_location": 7
                }
            }
        '
    ),
    ('Lawrenceville','MHWP',
        '
            {
                "visit": {
                    "mhwp_location_visit": 1
                },
                "participant": {
                    "mhwp_location": 1
                }
            }
        '
    ),
    ('Petersburg', 'MHWP',
        '
            {
                "visit": {
                    "mhwp_location_visit": 2
                },
                "participant": {
                    "mhwp_location": 2
                }
            }
        '
    ),
    ('Tappahannock', 'MHWP',
        '
            {
                "visit": {
                    "mhwp_location_visit": 3
                },
                "participant": {
                    "mhwp_location": 3
                }
            }
        '
    ),
    ('Southwood', 'MHWP',
        '
            {
                "visit": {
                    "mhwp_location_visit": 4
                },
                "participant": {
                    "mhwp_location": 4
                }
            }
        '
    );



-- Take the primary key from petersburg and insert setting the parent location
-- VCRC
-- Police substation
-- Gilhaven
-- VSU Van
INSERT INTO locations(name, program, parent_location, red_cap_connection_rules) VALUES
    ('VCRC', 'MHWP', (SELECT id FROM locations WHERE name = 'Petersburg' LIMIT 1),
        '
            {

                "visit": {
                    "mhwp_location_visit": 2,
                    "mhwp_location_visit_petersburg": 1
                },
                "participant": {
                    "mhwp_location": 2,
                    "mhwp_location_petersburg": 1
                }
            }
        '
    ),
    ('Police substation', 'MHWP', (SELECT id FROM locations WHERE name = 'Petersburg' LIMIT 1),
        '
            {
                "visit": {
                    "mhwp_location_visit": 2,
                    "mhwp_location_visit_petersburg": 2
                },
                "participant": {
                    "mhwp_location": 2,
                    "mhwp_location_petersburg": 2
                }
            }
        '
    ),
    ('Gilhaven', 'MHWP', (SELECT id FROM locations WHERE name = 'Petersburg' LIMIT 1),
        '
            {
                "visit": {
                    "mhwp_location_visit": 2,
                    "mhwp_location_visit_petersburg": 3
                },
                "participant": {
                    "mhwp_location": 2,
                    "mhwp_location_petersburg": 3
                }
            }
        '
    ),
    ('VSU Van', 'MHWP', (SELECT id FROM locations WHERE name = 'Petersburg' LIMIT 1),
        '
            {
                "visit": {
                    "mhwp_location_visit": 2,
                    "mhwp_location_visit_petersburg": 4
                },
                "participant": {
                    "mhwp_location": 2,
                    "mhwp_location_petersburg": 4
                }
            }
        '
    );


CREATE TABLE question_categories(
    id serial PRIMARY KEY,
    form VARCHAR(255) NOT NULL,
    string_id VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT
);
CREATE TABLE questions(
    id serial PRIMARY KEY,
    category_id INTEGER,
        CONSTRAINT FK_questions_category
            FOREIGN KEY (category_id)
            REFERENCES question_categories(id)
            ON DELETE CASCADE,
    string_id VARCHAR(255) UNIQUE,
    string_id_other VARCHAR(255) UNIQUE,
    question_type VARCHAR(255) NOT NULL,
    question VARCHAR(255) NOT NULL,
    description TEXT,
    required BOOLEAN DEFAULT FALSE,
    removed BOOLEAN DEFAULT FALSE,
    requirements TEXT,
    additional_options JSONB
);
CREATE TABLE IF NOT EXISTS question_options(
    id serial PRIMARY KEY,
    question_id INTEGER NOT NULL,
        CONSTRAINT FK_question_options_question_id
            FOREIGN KEY (question_id)
            REFERENCES questions(id)
            ON DELETE CASCADE,
    string_id VARCHAR(255),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    red_cap_option_index INTEGER,
    removed BOOLEAN NOT NULL DEFAULT FALSE,
    additional_options JSONB
);