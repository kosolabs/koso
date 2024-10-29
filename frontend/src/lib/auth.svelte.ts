import { jwtDecode } from "jwt-decode";

const CREDENTIAL_KEY = "credential";

export type User = {
  email: string;
  name: string;
  picture: string;
  exp: number;
};

class Auth {
  #token: string | null = $state(localStorage.getItem(CREDENTIAL_KEY) || null);
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

  set token(token: string) {
    this.#token = token;
    localStorage.setItem(CREDENTIAL_KEY, token);
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
    this.#token = null;
    localStorage.removeItem(CREDENTIAL_KEY);
  }
}
export const auth = new Auth();
