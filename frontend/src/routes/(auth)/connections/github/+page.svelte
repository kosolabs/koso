<script lang="ts">
  import Navbar from "$lib/navbar.svelte";
  import { headers, parse_response as parseResponse } from "$lib/api";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { goto } from "$app/navigation";
  import type internal from "stream";

  const prodClientId = "Iv23lioB8K1C62NP3UbV";
  const devClientId = "Iv23lif5pPjNjiQVtgPH";
  const stateSessionKey = "github_csrf_state";
  const loginExpirationSessionKey = "github_login_expires_at";

  type AuthResult = {
    expiresIn: number;
  };

  type InstallationsResponse = {
    installations: Installation[];
  };

  type Installation = {
    installationId: string;
    name: string;
  };

  type ConnectResponse = {};

  onMount(async () => {
    // See https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#using-the-web-application-flow-to-generate-a-user-access-token
    const urlParams = new URLSearchParams(window.location.search);

    let state = urlParams.get("state");
    if (!state) {
      state = `csrf_${Math.random().toString(36).substring(2)}`;
    }

    let expiresAt = sessionStorage.getItem(loginExpirationSessionKey);
    let validLogin =
      expiresAt && parseInt(expiresAt) - Date.now() > 30 * 60 * 1000;
    const code = urlParams.get("code");
    if (!validLogin && !code) {
      if (!validLogin) {
        console.log(
          `Login expired at ${expiresAt}: remaining ${expiresAt && parseInt(expiresAt) - Date.now() > 30 * 60 * 1000}`,
        );
      }
      redirectToGithubOAuthUrl(state);
      return;
    }

    const installationId = urlParams.get("installation_id");
    console.log(`Working on installation ${installationId} and state ${state}`);

    if (!validateCsrfState(state)) {
      toast.error(
        "Something went wrong connecting to Github. Please try again",
      );
      await goto("/");
      return;
    }

    if (code) {
      await loginWithCode(code);
      console.log("Logged in");
    } else {
      console.log("Already logged in");
    }

    const installations = await fetchInstallations();
    console.log(`Got installations ${installations}`);
    if (installationId) {
      const installation = installations.installations.find(
        (inst) => inst.installationId === installationId,
      );
      if (!installation) {
        console.log("user does not have access to installation");
        return;
      } else {
        console.log(`Found installation: ${installation}`);
      }
    }

    let prefix = "project_";
    if (state.startsWith(prefix)) {
      const projectId = state.substring(prefix.length);
      console.log(`WOrking on project: ${projectId}`);

      if (installationId) {
        console.log(
          `Connecting project '${projectId}' to installation '${installationId}''`,
        );
        await connectProject(projectId, installationId);
      }
    } else {
      console.log("Select a project");
    }
  });

  function redirectToGithubOAuthUrl(state: string) {
    const url = new URL("https://github.com/login/oauth/authorize");
    url.searchParams.append("client_id", devClientId);
    const redirectUri = `${location.origin}/connections/github`;
    url.searchParams.append("redirect_uri", redirectUri);
    sessionStorage.setItem(stateSessionKey, state);
    url.searchParams.append("state", state);

    console.log(
      `Redirecting to github oauth with redirect_uri ${redirectUri} and state ${state}`,
    );
    window.location.replace(url);
  }

  function validateCsrfState(state: string | null): boolean {
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

  async function loginWithCode(code: string): Promise<void> {
    let response = await fetch(`/plugins/github/auth`, {
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
    let expiresAt = Math.floor(Date.now()) + (result.expiresIn - 60) * 1000;
    sessionStorage.setItem(loginExpirationSessionKey, expiresAt.toString());
  }

  async function fetchInstallations(): Promise<InstallationsResponse> {
    let response = await fetch(`/plugins/github/installations`, {
      method: "GET",
      headers: headers(),
    });
    return parseResponse(response);
  }

  async function connectProject(
    projectId: string,
    installationId: string,
  ): Promise<ConnectResponse> {
    let response = await fetch(`/plugins/github/connect`, {
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
    return await parseResponse(response);
  }
</script>

<Navbar />

<div></div>
