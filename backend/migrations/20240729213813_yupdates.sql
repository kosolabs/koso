CREATE TABLE yupdates (
  project_id VARCHAR(36) NOT NULL,
  seq SERIAL NOT NULL,
  update_v2 BYTEA NOT NULL,
  PRIMARY KEY (project_id, seq)
);

DROP TABLE tasks;
