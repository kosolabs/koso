import { parse_response } from "./api";
import { auth, type User } from "./auth.svelte";

export type Project = {
  project_id: string;
  name: string;
};

export type UpdateProjectUsers = {
  project_id: string;
  add_emails: string[];
  remove_emails: string[];
};

export type ProjectExport = {
  project_id: string;
  data: unknown;
};

export const COMPARE_USERS_BY_NAME_AND_EMAIL = (a: User, b: User) =>
  a.name.localeCompare(b.name) || a.email.localeCompare(b.email);

export async function fetchProjects(): Promise<Project[]> {
  const response = await fetch("/api/projects", {
    method: "GET",
    headers: auth.headers(),
  });
  return parse_response(response);
}

export async function fetchProject(projectId: string): Promise<Project> {
  const response = await fetch(`/api/projects/${projectId}`, {
    method: "GET",
    headers: auth.headers(),
  });
  return parse_response(response);
}

export async function createProject(): Promise<Project> {
  const response = await fetch("/api/projects", {
    method: "POST",
    headers: {
      ...auth.headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ name: "My Project!" }),
  });
  return parse_response(response);
}

export async function updateProject(project: Project): Promise<Project> {
  const response = await fetch(`/api/projects/${project.project_id}`, {
    method: "PATCH",
    headers: {
      ...auth.headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify(project),
  });
  return parse_response(response);
}

export async function fetchProjectUsers(projectId: string): Promise<User[]> {
  const response = await fetch(`/api/projects/${projectId}/users`, {
    headers: auth.headers(),
  });
  const users: User[] = await parse_response(response);
  users.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
  return users;
}

export async function updateProjectUsers(
  update: UpdateProjectUsers,
): Promise<void> {
  const response = await fetch(`/api/projects/${update.project_id}/users`, {
    method: "PATCH",
    headers: {
      ...auth.headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify(update),
  });
  await parse_response(response);
}

export async function exportProject(projectId: string): Promise<ProjectExport> {
  const response = await fetch(`/api/projects/${projectId}/export`, {
    headers: auth.headers(),
  });
  return await parse_response(response);
}
