import { headers, parseResponse } from "$lib/api";

export type User = {
  email: string;
  name: string;
  picture: string;
};

export type FullUser = User & {
  premium: boolean;
};

export async function fetchUser(email: string): Promise<FullUser> {
  const response = await fetch(`/api/users/${email}`, {
    method: "GET",
    headers: headers(),
  });
  return parseResponse(response);
}
