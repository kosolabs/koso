import { headers, parseResponse } from "$lib/api";
import type { AuthContext } from "./auth.svelte";

export type User = {
  email: string;
  name: string;
  picture: string;
};

export type FullUser = User & {
  premium: boolean;
};

export async function fetchUser(
  auth: AuthContext,
  email: string,
): Promise<FullUser> {
  const response = await fetch(`/api/users/${email}`, {
    method: "GET",
    headers: headers(auth),
  });
  return parseResponse(auth, response);
}
