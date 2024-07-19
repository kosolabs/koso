import { googleLogout } from "google-oauth-gsi";
import { jwtDecode } from "jwt-decode";
import { derived, writable } from "svelte/store";

export type User = {
  email: string;
  name: string;
  picture: string;
  exp: number;
};

export const CREDENTIAL_KEY = "credential";

export const token = writable<string | null>(
  sessionStorage.getItem(CREDENTIAL_KEY) || null,
);

token.subscribe((token) => {
  if (token === null) {
    googleLogout();
    sessionStorage.removeItem(CREDENTIAL_KEY);
  } else {
    sessionStorage.setItem(CREDENTIAL_KEY, token);
  }
});

export function logout() {
  token.set(null);
}

export const user = derived(token, (token) => {
  if (token === null) {
    return null;
  }
  const user = jwtDecode(token) as User;
  if (user.exp * 1000 < Date.now()) {
    return null;
  }
  return user;
});
