-- Participants Table.
CREATE TABLE IF NOT EXISTS participants(
    -- serial 64 auto incrementing
    id serial PRIMARY KEY,
    red_cap_id INTEGER UNIQUE,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    -- Contact Info
    phone_number_one VARCHAR(255) ,
    phone_number_two VARCHAR(255) ,
    other_contact TEXT,
    -- Other Info
    program VARCHAR(255) NOT NULL,
    -- Relates to location table
    location INTEGER,
        CONSTRAINT FK_participants_location
            FOREIGN KEY (location)
            REFERENCES locations(id)
            ON DELETE SET NULL,
    status VARCHAR(255),
    behavioral_risks_identified TEXT,
    date_care_coordination_consent_signed DATE,
    date_home_visit_consent_signed DATE,
    signed_up_on DATE DEFAULT CURRENT_DATE,
    added_to_db_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_synced_with_redcap TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS participant_demographics(
    -- serial 64 auto incrementing
    id serial PRIMARY KEY,
    participant_id integer NOT NULL,
    -- Relates to participants table
        CONSTRAINT FK_participant_demographics_participant_id
            FOREIGN KEY (participant_id)
            REFERENCES participants(id)
            ON DELETE CASCADE,
    age smallint,
    gender TEXT,
    race VARCHAR(255)[],
    race_other TEXT,
    race_multiple TEXT,
    ethnicity VARCHAR(255),
    language TEXT,
    is_veteran BOOLEAN,
    health_insurance VARCHAR(255)[],
    highest_education_level VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS participant_health_overview(
    -- serial 64 auto incrementing
    id serial PRIMARY KEY,
    participant_id INTEGER NOT NULL,
    -- Relates to participants table
        CONSTRAINT FK_participant_health_overview_participant_id
            FOREIGN KEY (participant_id)
            REFERENCES participants(id)
            ON DELETE CASCADE,
    height integer,
    reported_health_conditions TEXT,
    allergies TEXT,
    has_blood_pressure_cuff BOOLEAN,
    takes_more_than_5_medications BOOLEAN,
    mobility_devices VARCHAR(255)[]
);

CREATE TABLE IF NOT EXISTS participant_medications(
    id serial PRIMARY KEY,
    participant_id INTEGER NOT NULL,
    -- Relates to participants table
        CONSTRAINT FK_participant_medications_participant_id
            FOREIGN KEY (participant_id)
            REFERENCES participants(id)
            ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    dosage VARCHAR(255),
    frequency TEXT,
    date_prescribed DATE,
    date_stopped_taking DATE,
    date_entered_into_system DATE DEFAULT CURRENT_DATE,
    is_current BOOLEAN,
    date_discontinued DATE,
    comments TEXT,
    -- Following Fields are for RedCap Sync and Internal Use
    red_cap_index INTEGER,
    hidden_from_red_cap BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS participant_goals(
    id serial PRIMARY KEY,
    participant_id INTEGER NOT NULL,
    -- Relates to participants table
        CONSTRAINT FK_participant_medical_history_participant_id
            FOREIGN KEY (participant_id)
            REFERENCES participants(id)
            ON DELETE CASCADE,
    goal TEXT NOT NULL,
    is_active BOOLEAN,
    -- Following Fields are for RedCap Sync and Internal Use
    red_cap_index INTEGER,
    hidden_from_red_cap BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS participant_goal_steps(
    id serial PRIMARY KEY,
    participant_id INTEGER NOT NULL,
    -- Relates to participants table
        CONSTRAINT FK_participant_medical_history_participant_id
            FOREIGN KEY (participant_id)
            REFERENCES participants(id)
            ON DELETE CASCADE,
    goal_id INTEGER,
    -- Relates to participant_goals table
    -- However, this is not required due to some weird logic within in RedCap
        CONSTRAINT FK_participant_goal_steps_goal_id
            FOREIGN KEY (goal_id)
            REFERENCES participant_goals(id)
            ON DELETE SET NULL,
    step TEXT NOT NULL,
    confidence_level smallint,
    date_set DATE,
    date_to_be_completed DATE,
    action_step BOOLEAN,
    -- Following Fields are for RedCap Sync and Internal Use
    red_cap_index INTEGER,
    hidden_from_red_cap BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS participant_question_answers(
    id bigserial PRIMARY KEY,
    participant_id INTEGER NOT NULL,
        CONSTRAINT FK_participant_question_answers_participant_id
            FOREIGN KEY (participant_id)
            REFERENCES participants(id)
            ON DELETE CASCADE,
    question_id integer NOT NULL,
        CONSTRAINT FK_participant_question_answers_question_id
            FOREIGN KEY (question_id)
            REFERENCES questions(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    -- Ensure that the combination of participant_id and question_id is unique
    UNIQUE (participant_id, question_id),
    value_radio INTEGER,
        CONSTRAINT FK_case_note_question_answers_value_radio
            FOREIGN KEY (value_radio)
            REFERENCES question_options(id)
            ON UPDATE CASCADE
            ON DELETE SET NULL,
    value_text TEXT,
    value_number INTEGER,
    value_boolean BOOLEAN,
    value_float REAL
);

CREATE TABLE IF NOT EXISTS participant_question_answer_mcb(
    id bigserial PRIMARY KEY,
    question_answers_id integer NOT NULL,
        CONSTRAINT FK_participant_question_answer_mcb_question_answers_id
            FOREIGN KEY (question_answers_id)
            REFERENCES participant_question_answers(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    option_id bigint NOT NULL,
        CONSTRAINT FK_participant_question_answer_mcb_option_id
            FOREIGN KEY (option_id)
            REFERENCES question_options(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);