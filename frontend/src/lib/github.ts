import { headers, parse_response } from "./api";

const stateSessionKey = "github_csrf_state";
const loginExpirationSessionKey = "github_login_expires_at";

export type AuthResult = {
  expiresIn: number;
};

export function githubInstallUrl(projectId: string) {
  return `https://github.com/apps/development-koso/installations/new?state=${encodeProjectIdCsrfState(projectId)}`;
}

export function redirectToGitubOAuth(state: string) {
  // const prodClientId = "Iv23lioB8K1C62NP3UbV";
  const devClientId = "Iv23lif5pPjNjiQVtgPH";
  const url = new URL("https://github.com/login/oauth/authorize");
  url.searchParams.append("client_id", devClientId);
  const redirectUri = `${location.origin}/connections/github`;
  url.searchParams.append("redirect_uri", redirectUri);
  url.searchParams.append("state", state);
  sessionStorage.setItem(stateSessionKey, state);
  console.log(`Redirecting to github oauth: ${url.toString()}`);
  window.location.replace(url);
}

export function generateCsrfState(): string {
  return `csrf_${Math.random().toString(36).substring(2)}`;
}

export function encodeProjectIdCsrfState(projectId: string): string {
  return `project_${projectId}`;
}

export function decodeCsrfStateAsProjectId(state: string): string | null {
  const prefix = "project_";
  if (state.startsWith(prefix)) {
    return state.substring(prefix.length);
  }
  return null;
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
