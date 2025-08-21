import { headers, parseResponse } from "$lib/api";
import type { FullUser, User } from "$lib/users";
import type { Graph } from "$lib/yproxy";
import type { AuthContext } from "./auth.svelte";

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

export type Dupe = {
  dupeId: string;
  projectId: string;
  task1Id: string;
  task2Id: string;
  similarity: string;
  // taskID: string;
  // taskName: string;
  // taskDescription: string;
  // parentTask: string;
};

export const COMPARE_USERS_BY_NAME_AND_EMAIL = (a: User, b: User) =>
  a.name.localeCompare(b.name) || a.email.localeCompare(b.email);

export async function fetchProjects(
  auth: AuthContext,
  filterDeleted: boolean = true,
): Promise<Project[]> {
  const response = await fetch("/api/projects", {
    method: "GET",
    headers: headers(auth),
  });
  let projects = await parseResponse<Project[]>(auth, response);
  if (filterDeleted) {
    projects = projects.filter((p) => !p.deletedOn);
  }
  return projects;
}

export async function fetchProject(
  auth: AuthContext,
  projectId: string,
): Promise<Project> {
  const response = await fetch(`/api/projects/${projectId}`, {
    method: "GET",
    headers: headers(auth),
  });
  return parseResponse(auth, response);
}

export async function createProject(
  auth: AuthContext,
  projectExport: ProjectExport | null = null,
): Promise<Project> {
  const response = await fetch("/api/projects", {
    method: "POST",
    headers: {
      ...headers(auth),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ name: "My Project!", projectExport }),
  });
  return parseResponse(auth, response);
}

export async function updateProject(
  auth: AuthContext,
  project: Project,
): Promise<Project> {
  const response = await fetch(`/api/projects/${project.projectId}`, {
    method: "PATCH",
    headers: {
      ...headers(auth),
      "Content-Type": "application/json",
    },
    body: JSON.stringify(project),
  });
  return parseResponse(auth, response);
}

export async function deleteProject(
  auth: AuthContext,
  project: Project,
): Promise<Project> {
  const response = await fetch(`/api/projects/${project.projectId}`, {
    method: "DELETE",
    headers: headers(auth),
  });
  return parseResponse(auth, response);
}

export async function fetchProjectUsers(
  auth: AuthContext,
  projectId: string,
): Promise<FullUser[]> {
  const response = await fetch(`/api/projects/${projectId}/users`, {
    headers: headers(auth),
  });
  const users: FullUser[] = await parseResponse(auth, response);
  users.sort(COMPARE_USERS_BY_NAME_AND_EMAIL);
  return users;
}

export async function updateProjectUsers(
  auth: AuthContext,
  update: UpdateProjectUsers,
): Promise<void> {
  const response = await fetch(`/api/projects/${update.projectId}/users`, {
    method: "PATCH",
    headers: {
      ...headers(auth),
      "Content-Type": "application/json",
    },
    body: JSON.stringify(update),
  });
  await parseResponse(auth, response);
}

export async function exportProject(
  auth: AuthContext,
  projectId: string,
): Promise<ProjectExport> {
  const response = await fetch(`/api/projects/${projectId}/export`, {
    headers: headers(auth),
  });
  return await parseResponse(auth, response);
}

export async function fetchDupes(
  auth: AuthContext,
  projectId: string,
): Promise<Dupe[]> {
  const response = await fetch(`/api/projects/${projectId}/dupes`, {
    headers: headers(auth),
  });
  if (!response.ok) {
    throw new Error(`Failed to fetch dupes: ${response.status}`);
  }

  // const data = await response.json();
  return await parseResponse(auth, response);
}

export async function updateDupes(
  auth: AuthContext,
  projectId: string,
  dupe_id: string,
  resolution: boolean | null,
): Promise<void> {
  const response = await fetch(`/api/projects/${projectId}/dupes/${dupe_id}`, {
    method: "PATCH",
    headers: {
      ...headers(auth),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ resolution }),
  });
  await parseResponse(auth, response);
}
