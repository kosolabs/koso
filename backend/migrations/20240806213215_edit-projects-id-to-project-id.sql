alter table projects drop constraint projects_pkey;
alter table projects rename column id TO project_id;
alter table tasks add constraint projects_pkey primary key (project_id);