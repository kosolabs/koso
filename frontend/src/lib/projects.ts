import type { User } from "./auth";
import { logout_on_authentication_error } from "./errors";

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

export async function fetchProjects(token: string | null): Promise<Project[]> {
  const response = await fetch("/api/projects", {
    method: "GET",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  if (!response.ok) {
    logout_on_authentication_error(response);
    throw new Error(
      `Failed to fetch projects: ${response.statusText} (${response.status})`,
    );
  }
  return await response.json();
}

export async function fetchProject(
  token: string | null,
  projectId: string,
): Promise<Project> {
  const response = await fetch(`/api/projects/${projectId}`, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  if (!response.ok) {
    logout_on_authentication_error(response);
    throw new Error(
      `Failed to fetch project: ${response.statusText} (${response.status})`,
    );
  }
  return await response.json();
}

export async function createProject(token: string | null): Promise<Project> {
  const response = await fetch("/api/projects", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ name: "My Project!" }),
  });
  if (!response.ok) {
    logout_on_authentication_error(response);
    throw new Error(
      `Failed to create project: ${response.statusText} (${response.status})`,
    );
  }
  return await response.json();
}

export async function updateProject(
  token: string | null,
  project: Project,
): Promise<Project> {
  const response = await fetch(`/api/projects/${project.project_id}`, {
    method: "PATCH",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(project),
  });
  if (!response.ok) {
    logout_on_authentication_error(response);
    throw new Error(
      `Failed to update project name: ${response.statusText} (${response.status})`,
    );
  }
  return await response.json();
}

export async function fetchProjectUsers(
  token: string | null,
  projectId: string,
): Promise<User[]> {
  const response = await fetch(`/api/projects/${projectId}/users`, {
    headers: { Authorization: "Bearer " + token },
  });
  if (!response.ok) {
    logout_on_authentication_error(response);
    throw new Error(
      `Failed to fetch project users: ${response.statusText} (${response.status})`,
    );
  }
  const users: User[] = await response.json();
  users.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
  return users;
}

export async function updateProjectUsers(
  token: string | null,
  update: UpdateProjectUsers,
): Promise<void> {
  const response = await fetch(`/api/projects/${update.project_id}/users`, {
    method: "PATCH",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(update),
  });
  if (!response.ok) {
    logout_on_authentication_error(response);
    throw new Error(
      `Failed to update project permissions: ${response.statusText} (${response.status})`,
    );
  }
}

export async function exportProject(
  token: string | null,
  projectId: string,
): Promise<ProjectExport> {
  const response = await fetch(`/api/projects/${projectId}/export`, {
    headers: { Authorization: "Bearer " + token },
  });
  if (!response.ok) {
    logout_on_authentication_error(response);
    throw new Error(
      `Failed to fetch project users: ${response.statusText} (${response.status})`,
    );
  }
  const projectExport: ProjectExport = await response.json();
  return projectExport;
}
