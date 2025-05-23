import { goto } from "$app/navigation";
import { headers, parseResponse } from "./api";

const stateSessionKey = "github_csrf_state";
const loginExpirationSessionKey = "github_login_expires_at";

export type InitResponse = {
  clientId: string;
  appName: string;
};

export type AuthResult = {
  expiresIn: number;
};

/**
 * Craft an install link that will redirect back to this project after install.
 * See
 * https://docs.github.com/en/apps/sharing-github-apps/sharing-your-github-app
 */
export async function githubInstallUrl(projectId: string) {
  const init = await initGithub();

  const state = encodeState<
    Omit<ConnectProjectState, "installationId"> & { installationId?: string }
  >({
    csrf: generateCsrfValue(),
    projectId: projectId,
    clientId: init.clientId,
    installationId: undefined,
  });
  sessionStorage.setItem(stateSessionKey, state);
  return `https://github.com/apps/${init.appName}/installations/new?state=${encodeURIComponent(state)}`;
}

/**
 * Craft a connect user link that will redirect back to the profile page on
 * success. See
 * https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#using-the-web-application-flow-to-generate-a-user-access-token
 */
export async function redirectToConnectUserFlow() {
  const init = await initGithub();

  const state = encodeState<ConnectUserState>({
    csrf: generateCsrfValue(),
    clientId: init.clientId,
  });
  sessionStorage.setItem(stateSessionKey, state);

  await goto(
    `${location.origin}/connections/github/user?state=${encodeURIComponent(state)}`,
  );
}

async function initGithub(): Promise<InitResponse> {
  const response = await fetch(`/plugins/github/init`, {
    method: "GET",
    headers: {
      ...headers(),
      "Content-Type": "application/json",
    },
  });
  return await parseResponse(response);
}

/**
 * Redirect to Github for OAuth autorization. After authorization, Github will
 * redirect back to us at the given redirect URI with the given state and a code
 * parameter.
 *
 * See
 * https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#generating-a-user-access-token-when-a-user-installs-your-app
 */
export function redirectToGitubOAuth<T extends BaseState>(
  state: T,
  redirectUri: string,
) {
  const url = new URL("https://github.com/login/oauth/authorize");
  url.searchParams.append("client_id", state.clientId);
  url.searchParams.append("redirect_uri", redirectUri);
  const stateStr = encodeState<T>(state);
  url.searchParams.append("state", stateStr);
  sessionStorage.setItem(stateSessionKey, stateStr);
  console.log(`Redirecting to github oauth: ${url.toString()}`);
  window.location.replace(url);
}

export type BaseState = {
  csrf: string;
  clientId: string;
};

export type ConnectProjectState = BaseState & {
  projectId: string;
  installationId: string;
};

export type ConnectUserState = BaseState & {};

function generateCsrfValue(): string {
  return `csrf_${Math.random().toString(36).substring(2)}`;
}

export function encodeState<T extends BaseState>(state: T): string {
  return btoa(JSON.stringify(state));
}

export function decodeState<T extends BaseState>(state: string): Partial<T> {
  return JSON.parse(atob(state));
}

export function validateStateForCsrf(state: string): boolean {
  const sessionCsrfState = sessionStorage.getItem(stateSessionKey);
  sessionStorage.removeItem(stateSessionKey);
  if (!sessionCsrfState) {
    console.warn(`CSRF state not present in session storage. Got: '${state}'`);
    return false;
  }
  if (sessionCsrfState !== state) {
    console.warn(
      `CSRF state mismatch. Expected:'${sessionCsrfState}', got:'${state}'`,
    );
    return false;
  }
  console.log("Validated CSRF session state");
  return true;
}

/**
 * Call the Koso backend to exchange the Github OAuth code for a user access
 * token.
 *
 * See
 * https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#generating-a-user-access-token-when-a-user-installs-your-app
 */
export async function authWithCode(code: string): Promise<void> {
  const response = await fetch(`/plugins/github/auth`, {
    method: "POST",
    headers: {
      ...headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      code: code,
    }),
  });
  const result: AuthResult = await parseResponse(response);
  const expiresAt = Math.floor(Date.now()) + (result.expiresIn - 60) * 1000;
  sessionStorage.setItem(loginExpirationSessionKey, expiresAt.toString());
}

export async function connectProject(
  projectId: string,
  installationId: string,
): Promise<void> {
  const response = await fetch(`/plugins/github/connect`, {
    method: "POST",
    headers: {
      ...headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      projectId,
      installationId,
    }),
  });
  await parseResponse(response);
  return;
}

export async function connectUser(): Promise<void> {
  const response = await fetch(`/plugins/github/connectUser`, {
    method: "POST",
    headers: {
      ...headers(),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({}),
  });
  await parseResponse(response);
  return;
}
