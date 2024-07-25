import { writable } from "svelte/store";
import { page } from "$app/stores";
import { get } from "svelte/store";

const ON_LOGIN_REDIRECT_KEY = "login-redirect";
const DO_NOT_REDIRECT = "DO_NOT";
const LAST_VISITED_PROJECT_KEY = "last-visited-project";

// The most recently viewed project or null.
export const lastVisitedProjectId = writable<string | null>(
  localStorage.getItem(LAST_VISITED_PROJECT_KEY) || null,
);

lastVisitedProjectId.subscribe((projectId) => {
  if (projectId === null) {
    localStorage.removeItem(LAST_VISITED_PROJECT_KEY);
  } else {
    localStorage.setItem(LAST_VISITED_PROJECT_KEY, projectId);
  }
});

// The path to redirect to on next login or
// DO_NOT_REDIRECT, set when the user explicitly logged out.
const onLoginRedirect = writable<string | null>(
  sessionStorage.getItem(ON_LOGIN_REDIRECT_KEY) || null,
);

onLoginRedirect.subscribe((redirect) => {
  if (redirect === null) {
    sessionStorage.removeItem(ON_LOGIN_REDIRECT_KEY);
  } else {
    sessionStorage.setItem(ON_LOGIN_REDIRECT_KEY, redirect);
  }
});

// Call this function when a user explicitly logs out
// in order to prevent setRedirectOnUserNotAuthenticated from
// redirecting the user back to where they were on login.
export function disableRedirectOnLogOut() {
  console.log("Disabling redirect on log out.");
  onLoginRedirect.update(() => DO_NOT_REDIRECT);
}

// Call this when a user becomes non-authenticated,
// whether by accessing a page without being logged in
// or their credentials expiring.
export function setRedirectOnUserNotAuthenticated() {
  if (get(onLoginRedirect) === DO_NOT_REDIRECT) {
    onLoginRedirect.update(() => null);
    console.log(
      "User isn't logged in and DO_NOT_REDIRECT is set. Going to / without a redirect destination.",
    );
    return;
  }

  const redirect = get(page).url.pathname;
  onLoginRedirect.update(() => redirect);
  console.log(
    `User isn't logged in. Going to / with redirect destination ${redirect}`,
  );
}

export function popRedirectOnLogin(): string | null {
  const redirect = get(onLoginRedirect);
  onLoginRedirect.update(() => null);

  if (!redirect || redirect === DO_NOT_REDIRECT) {
    return "";
  }
  return redirect;
}
