CREATE TABLE dedupe_candidates (
    dupe_id varchar(22) PRIMARY KEY,
    project_id varchar(36) NOT NULL,
    task_1_id varchar(255) NOT NULL,
    task_2_id varchar(255) NOT NULL,
    similarity decimal(17,16) NOT NULL,
    detected_at timestamptz NOT NULL DEFAULT NOW(),
    resolution boolean,
    resolved_at timestamptz,
    resolved_by varchar(320),
    UNIQUE(project_id, task_1_id, task_2_id)
);
