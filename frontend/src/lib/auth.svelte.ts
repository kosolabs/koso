import { goto } from "$app/navigation";
import { nav } from "$lib/nav.svelte";
import { jwtDecode } from "jwt-decode";
import { CircleX } from "lucide-svelte";
import { getContext, setContext } from "svelte";
import { getDialoguerContext } from "./kosui/dialog";
import { loads, saves } from "./stores.svelte";
import { fetchUser, type FullUser } from "./users";

export const CREDENTIAL_KEY = "credential";

type User = {
  email: string;
  name: string;
  picture: string;
  exp: number;
};
export class AuthContext {
  #fullUser: FullUser | undefined = $state();

  constructor() {}

  get fullUser(): FullUser | undefined {
    return this.#fullUser;
  }

  async load() {
    if (this.ok()) {
      this.#fullUser = await fetchUser(this.user.email);
    }
  }

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
    return this.#token !== null && this.#user !== null;
  }

  headers() {
    return { Authorization: `Bearer ${this.token}` };
  }

  logout() {
    this.token = null;
  }
}

export async function showUnauthorizedDialog() {
  const dialog = getDialoguerContext();
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

export function setAuthContext(ctx: AuthContext): AuthContext {
  return setContext<AuthContext>(AuthContext, ctx);
}

export function getAuthContext(): AuthContext {
  const ctx = getContext<AuthContext>(AuthContext);
  if (!ctx) throw new Error("AuthContext is undefined");
  return ctx;
}
