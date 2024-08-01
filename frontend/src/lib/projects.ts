export type ProjectUsers = {
  [email: string]: {
    name: string;
    email: string;
    picture: string;
  };
};

export type Project = {
  project_id: string;
  name: string;
};

export async function fetchProjects(token: string | null): Promise<Project[]> {
  const response = await fetch("/api/projects", {
    method: "GET",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  if (!response.ok) {
    throw new Error(
      `Failed to fetch projects: ${response.statusText} (${response.status})`,
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
    throw new Error(
      `Failed to create project: ${response.statusText} (${response.status})`,
    );
  }
  return await response.json();
}
