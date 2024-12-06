import { headers, parse_response } from "./api";

const stateSessionKey = "github_csrf_state";
const loginExpirationSessionKey = "github_login_expires_at";

export type AuthResult = {
  expiresIn: number;
};

/**
 * Craft an install link that will redirect back to this project after install.
 * See
 * https://docs.github.com/en/apps/sharing-github-apps/sharing-your-github-app
 */
export function githubInstallUrl(projectId: string) {
  // TODO: Figure out how to make this work with static builds locally
  const app =
    import.meta.env.MODE === "production" ? "koso-github" : "development-koso";
  const state = encodeURIComponent(
    encodeState({
      csrf: generateCsrfState(),
      projectId: projectId,
    }),
  );
  return `https://github.com/apps/${app}/installations/new?state=${state}`;
}

/**
 * Redirect to Github for OAuth autorization. After authorization, Github will
 * redirect back to us at our redirect URI: /connections/github with the given
 * state and a code parameter.
 *
 * See
 * https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#generating-a-user-access-token-when-a-user-installs-your-app
 */
export function redirectToGitubOAuth(state: string) {
  const clientId =
    import.meta.env.MODE === "production"
      ? "Iv23lioB8K1C62NP3UbV"
      : "Iv23lif5pPjNjiQVtgPH";

  const url = new URL("https://github.com/login/oauth/authorize");
  url.searchParams.append("client_id", clientId);
  const redirectUri = `${location.origin}/connections/github`;
  url.searchParams.append("redirect_uri", redirectUri);
  url.searchParams.append("state", state);
  sessionStorage.setItem(stateSessionKey, state);
  console.log(`Redirecting to github oauth: ${url.toString()}`);
  window.location.replace(url);
}

export type State = {
  csrf?: string | null;
  projectId?: string | null;
  installationId?: string | null;
};

export function generateCsrfState(): string {
  return `csrf_${Math.random().toString(36).substring(2)}`;
}

export function encodeState(state: State): string {
  return btoa(JSON.stringify(state));
}

export function decodeState(state: string): State {
  return JSON.parse(atob(state));
}

export function validateCsrfState(state: string | null): boolean {
  const sessionCsrfState = sessionStorage.getItem(stateSessionKey);
  sessionStorage.removeItem(stateSessionKey);
  if (sessionCsrfState && state && sessionCsrfState !== state) {
    console.warn(
      `CSRF state mismatch. Expected:'${sessionCsrfState}'', got:'${state}''`,
    );
    return false;
  }
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
  const result: AuthResult = await parse_response(response);
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
  await parse_response(response);
  return;
}
