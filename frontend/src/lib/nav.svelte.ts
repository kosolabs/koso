import { page } from "$app/stores";
import { get } from "svelte/store";
import { loads, saves } from "./stores.svelte";

const ON_LOGIN_REDIRECT_KEY = "login-redirect";
const LAST_VISITED_PROJECT_KEY = "last-visited-project";

class Nav {
  #lastVisitedProjectId = $state(loads(LAST_VISITED_PROJECT_KEY, null));
  #onLoginRedirect = $state(
    loads(ON_LOGIN_REDIRECT_KEY, null, { storage: sessionStorage }),
  );

  get lastVisitedProjectId() {
    return this.#lastVisitedProjectId;
  }

  set lastVisitedProjectId(value: string | null) {
    this.#lastVisitedProjectId = value;
    saves(LAST_VISITED_PROJECT_KEY, value);
  }

  set onLoginRedirect(value: string | null) {
    this.#onLoginRedirect = value;
    saves(ON_LOGIN_REDIRECT_KEY, value, { storage: sessionStorage });
  }

  /**
   * Call this when a user becomes non-authenticated, whether by accessing a
   * page without being logged in or their credentials expiring.
   */
  pushRedirectOnUserNotAuthenticated() {
    const redirect = get(page).url.pathname;
    this.#onLoginRedirect = redirect;
    console.debug(
      `User isn't logged in. Going to / with redirect destination ${redirect}`,
    );
  }

  popRedirectOnLogin(): string {
    const redirect = this.#onLoginRedirect;
    this.#onLoginRedirect = null;
    return redirect || "";
  }
}
export const nav = new Nav();
