import { headers, parse_response as parseResponse } from "$lib/api";
import type { FullUser, User } from "$lib/users";
import type { Graph } from "$lib/yproxy";

export type Project = {
  projectId: string;
  name: string;
  deletedOn?: string;
};

export type UpdateProjectUsers = {
  projectId: string;
  addEmails: string[];
  removeEmails: string[];
};

export type ProjectExport = {
  projectId: string;
  graph: Graph;
};

export const COMPARE_USERS_BY_NAME_AND_EMAIL = (a: User, b: User) =>
  a.name.localeCompare(b.name) || a.email.localeCompare(b.email);

export async function fetchProjects(
  filterDeleted: boolean = true,
): Promise<Project[]> {
  const response = await fetch("/api/projects", {
    method: "GET",
    headers: headers(),
  });
  let projects = await parseResponse<Project[]>(response);
  if (filterDeleted) {
    projects = projects.filter((p) => !p.deletedOn);
  }
  return projects;
}

export async function fetchProject(projectId: string): Promise<Project> {
  const response = await fetch(`/api/projects/${projectId}`, {
    method: "GET",
    headers: headers(),
  });
  return parseResponse(response);
}

export async function createProject(
  projectExport: ProjectExport | null = null,
): Promise<Project> {
  const response = await fetch("/api/projects", {
    method: "POST",
    headers: {
      ...headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ name: "My Project!", projectExport }),
  });
  return parseResponse(response);
}

export async function updateProject(project: Project): Promise<Project> {
  const response = await fetch(`/api/projects/${project.projectId}`, {
    method: "PATCH",
    headers: {
      ...headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify(project),
  });
  return parseResponse(response);
}

export async function deleteProject(project: Project): Promise<Project> {
  const response = await fetch(`/api/projects/${project.projectId}`, {
    method: "DELETE",
    headers: headers(),
  });
  return parseResponse(response);
}

export async function fetchProjectUsers(
  projectId: string,
): Promise<FullUser[]> {
  const response = await fetch(`/api/projects/${projectId}/users`, {
    headers: headers(),
  });
  const users: FullUser[] = await parseResponse(response);
  users.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
  return users;
}

export async function updateProjectUsers(
  update: UpdateProjectUsers,
): Promise<void> {
  const response = await fetch(`/api/projects/${update.projectId}/users`, {
    method: "PATCH",
    headers: {
      ...headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify(update),
  });
  await parseResponse(response);
}

export async function exportProject(projectId: string): Promise<ProjectExport> {
  const response = await fetch(`/api/projects/${projectId}/export`, {
    headers: headers(),
  });
  return await parseResponse(response);
}
