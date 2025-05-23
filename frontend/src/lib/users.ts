import { headers, parseResponse } from "$lib/api";

export type User = {
  email: string;
  name: string;
  picture: string;
};

export type FullUser = User & {
  premium: boolean;
};

type UpdateUser = {
  githubLogin?: string;
};

export async function fetchUser(email: string): Promise<FullUser> {
  const response = await fetch(`/api/users/${email}`, {
    method: "GET",
    headers: headers(),
  });
  return parseResponse(response);
}

export async function updateUser(email: string, updateUser: UpdateUser) {
  const response = await fetch(`/api/users/${email}`, {
    method: "PATCH",
    headers: {
      ...headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify(updateUser),
  });
  return parseResponse(response);
}
