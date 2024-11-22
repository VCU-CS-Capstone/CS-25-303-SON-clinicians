-- Add up migration script here
CREATE TABLE IF NOT EXISTS case_notes(
    id serial PRIMARY KEY,
    participant_id integer NOT NULL,
        CONSTRAINT FK_participant_case_notes_participant_id
            FOREIGN KEY (participant_id)
            REFERENCES participants(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    location integer,
        CONSTRAINT FK_participant_case_notes_location
            FOREIGN KEY (location)
            REFERENCES locations(id)
            ON UPDATE CASCADE
            ON DELETE SET NULL,
    visit_type VARCHAR(255),
    age smallint,
    reason_for_visit TEXT,
    info_provided_by_caregiver TEXT,
    date_of_visit DATE NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    pushed_to_red_cap BOOLEAN,
    red_cap_instance integer,
    -- Ensure that the combination of participant_id and red_cap_instance is unique
    UNIQUE (participant_id, red_cap_instance),
    last_synced_with_redcap TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS case_note_health_measures(
    id serial PRIMARY KEY,
    case_note_id integer NOT NULL,
        CONSTRAINT FK_case_note_health_measures_case_note_id
            FOREIGN KEY (case_note_id)
            REFERENCES case_notes(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    weight real,
    glucose_tested BOOLEAN NOT NULL DEFAULT FALSE,
    glucose_result real,
    fasted_atleast_2_hours BOOLEAN,
    other TEXT
);

CREATE TABLE IF NOT EXISTS health_measure_blood_pressure(
    id bigserial PRIMARY KEY,
    health_measure_id bigint NOT NULL,
        CONSTRAINT FK_health_measure_multi_check_box_health_measure_id
            FOREIGN KEY (health_measure_id)
            REFERENCES case_note_health_measures(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    blood_pressure_type VARCHAR(255) NOT NULL,
    -- Ensure that the combination of health_measure_id and blood_pressure_type is unique
    UNIQUE (health_measure_id, blood_pressure_type),
    systolic INTEGER NOT NULL,
    diastolic INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS case_note_question_answers(
    id bigserial PRIMARY KEY,
    case_note_id integer NOT NULL,
        CONSTRAINT FK_case_note_question_answers_case_note_id
            FOREIGN KEY (case_note_id)
            REFERENCES case_notes(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    question_id integer NOT NULL,
        CONSTRAINT FK_case_note_question_answers_question_id
            FOREIGN KEY (question_id)
            REFERENCES questions(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    -- Ensure that the combination of case_note_id and question_id is unique
    UNIQUE (case_note_id, question_id),
    response_type VARCHAR(255) NOT NULL,
    value_radio INTEGER,
        CONSTRAINT FK_case_note_question_answers_value_radio
            FOREIGN KEY (value_radio)
            REFERENCES question_options(id)
            ON UPDATE CASCADE
            ON DELETE SET NULL,
    value_text TEXT,
    value_boolean BOOLEAN,
    value_number INTEGER,
    value_float REAL
);

CREATE TABLE IF NOT EXISTS case_note_question_answer_mcb(
    id bigserial PRIMARY KEY,
    question_answers_id bigint NOT NULL,
        CONSTRAINT FK_question_answer_multi_check_box_question_answers_id
            FOREIGN KEY (question_answers_id)
            REFERENCES case_note_question_answers(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE,
    option_id integer NOT NULL,
        CONSTRAINT FK_question_answer_multi_check_box_option_id
            FOREIGN KEY (option_id)
            REFERENCES question_options(id)
            ON UPDATE CASCADE
            ON DELETE CASCADE
);