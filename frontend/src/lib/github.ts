import { goto } from "$app/navigation";
import { headers, parseResponse } from "./api";
import type { AuthContext } from "./auth.svelte";

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
export async function redirectToGithubInstallFlow(
  auth: AuthContext,
  projectId: string,
  redirectUrl: string,
) {
  const init = await initGithub(auth);

  const state = encodeState<
    Omit<ConnectProjectState, "installationId"> & { installationId?: string }
  >({
    csrf: generateCsrfValue(),
    projectId,
    clientId: init.clientId,
    installationId: undefined,
    redirectUrl,
  });
  sessionStorage.setItem(stateSessionKey, state);
  window.location.assign(
    `https://github.com/apps/${init.appName}/installations/new?state=${encodeURIComponent(state)}`,
  );
}

/**
 * Craft a connect user link that will redirect back to the profile page on
 * success. See
 * https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#using-the-web-application-flow-to-generate-a-user-access-token
 */
export async function redirectToConnectUserFlow(
  auth: AuthContext,
  redirectUrl: string,
) {
  const init = await initGithub(auth);

  const state = encodeState<ConnectUserState>({
    csrf: generateCsrfValue(),
    clientId: init.clientId,
    redirectUrl,
  });
  sessionStorage.setItem(stateSessionKey, state);

  await goto(`/connections/github/user?state=${encodeURIComponent(state)}`);
}

async function initGithub(auth: AuthContext): Promise<InitResponse> {
  const response = await fetch(`/plugins/github/init`, {
    method: "GET",
    headers: {
      ...headers(auth),
      "Content-Type": "application/json",
    },
  });
  return await parseResponse(auth, response);
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
  redirectUrl: string;
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
export async function authWithCode(
  auth: AuthContext,
  code: string,
): Promise<void> {
  const response = await fetch(`/plugins/github/auth`, {
    method: "POST",
    headers: {
      ...headers(auth),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      code: code,
    }),
  });
  const result: AuthResult = await parseResponse(auth, response);
  const expiresAt = Math.floor(Date.now()) + (result.expiresIn - 60) * 1000;
  sessionStorage.setItem(loginExpirationSessionKey, expiresAt.toString());
}

export async function connectProject(
  auth: AuthContext,
  projectId: string,
  installationId: string,
): Promise<void> {
  const response = await fetch(`/plugins/github/connect`, {
    method: "POST",
    headers: {
      ...headers(auth),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      projectId,
      installationId,
    }),
  });
  await parseResponse(auth, response);
  return;
}

export async function connectUser(auth: AuthContext): Promise<void> {
  const response = await fetch(`/plugins/github/userConnections`, {
    method: "POST",
    headers: {
      ...headers(auth),
      "Content-Type": "application/json",
    },
    body: JSON.stringify({}),
  });
  await parseResponse(auth, response);
  return;
}

export async function deleteUserConnection(auth: AuthContext): Promise<void> {
  const response = await fetch(`/plugins/github/userConnections`, {
    method: "DELETE",
    headers: {
      ...headers(auth),
    },
  });
  await parseResponse(auth, response);
  return;
}
