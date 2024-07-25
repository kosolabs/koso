import { writable } from "svelte/store";

const ON_LOGIN_REDIRECT_KEY = "login-redirect";
export const DO_NOT_REDIRECT = "DO_NOT";
const LAST_VISITED_PROJECT_KEY = "last-visited-project";

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

export const onLoginRedirect = writable<string | null>(
  sessionStorage.getItem(ON_LOGIN_REDIRECT_KEY) || null,
);

onLoginRedirect.subscribe((redirect) => {
  if (redirect === null) {
    sessionStorage.removeItem(ON_LOGIN_REDIRECT_KEY);
  } else {
    sessionStorage.setItem(ON_LOGIN_REDIRECT_KEY, redirect);
  }
});
