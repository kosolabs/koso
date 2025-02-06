import { goto } from "$app/navigation";
import { dialog } from "$lib/kosui/dialog";
import { nav } from "$lib/nav.svelte";
import { jwtDecode } from "jwt-decode";
import { CircleX } from "lucide-svelte";
import { loads, saves } from "./stores.svelte";

export const CREDENTIAL_KEY = "credential";

export type User = {
  email: string;
  name: string;
  picture: string;
  exp: number;
};

class Auth {
  #token: string | null = $state(loads(CREDENTIAL_KEY, null));
  #user: User | null = $derived.by(() => {
    if (this.#token === null) {
      return null;
    }
    const user = jwtDecode(this.#token) as User;
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
        this.logout();
      },
      // Avoid exceeding setTimeout's max delay.
      Math.min(remainingLifeMillis - 90000, 2147483647),
    );
    return user;
  });

  get token(): string {
    if (!this.#token) throw new Error("Unauthenticated");
    return this.#token;
  }

  set token(token: string | null) {
    this.#token = token;
    saves(CREDENTIAL_KEY, token);
  }

  get user(): User {
    if (!this.#user) throw new Error("Unauthenticated");
    return this.#user;
  }

  ok(): boolean {
    return this.#token !== null;
  }

  headers() {
    return { Authorization: `Bearer ${this.token}` };
  }

  logout() {
    this.token = null;
  }
}
export const auth = new Auth();

export async function showUnauthorizedDialog() {
  await dialog.notice({
    icon: CircleX,
    title: "Unauthorized",
    message:
      "You do not have access to this project or the project does not exist.",
    acceptText: "Return Home",
  });
  nav.lastVisitedProjectId = null;
  await goto("/");
}
