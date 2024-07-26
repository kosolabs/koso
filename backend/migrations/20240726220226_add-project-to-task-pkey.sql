alter table tasks drop constraint tasks_pkey;
alter table tasks add constraint tasks_pkey primary key (project_id, id);