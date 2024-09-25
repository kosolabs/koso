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
  localStorage.getItem(CREDENTIAL_KEY) || null,
);

token.subscribe((token) => {
  if (token === null) {
    googleLogout();
    localStorage.removeItem(CREDENTIAL_KEY);
  } else {
    localStorage.setItem(CREDENTIAL_KEY, token);
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
  // Allow the token to last seven days longer than the given expiry.
  // This number matches the server's validation in google.rs.
  const sevenDaysSecs = 7 * 24 * 60 * 60;
  const realExpiryMillisecs = (user.exp + sevenDaysSecs) * 1000;
  const remainingLifeMillis = realExpiryMillisecs - Date.now();
  if (remainingLifeMillis <= 0) {
    return null;
  }
  setTimeout(
    () => {
      console.debug("Logging the user out at token expiry");
      logout();
    },
    // Avoid exceeding setTimeout's max delay.
    Math.min(remainingLifeMillis - 90000, 2147483647),
  );
  return user;
});
