alter table projects drop constraint projects_pkey;
alter table projects rename column id TO project_id;
alter table projects add constraint projects_pkey primary key (project_id);