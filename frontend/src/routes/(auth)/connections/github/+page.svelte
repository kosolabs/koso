<script lang="ts">
  import Navbar from "$lib/navbar.svelte";
  import { headers, parse_response as parseResponse } from "$lib/api";
  import { onMount } from "svelte";

  const prodClientId = "Iv23lioB8K1C62NP3UbV";
  const devClientId = "Iv23lif5pPjNjiQVtgPH";
  const stateSessionKey = "github_csrf_state";

  type AuthResult = {
    access_token: string;
  };

  onMount(async () => {
    // See https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#using-the-web-application-flow-to-generate-a-user-access-token
    const urlParams = new URLSearchParams(window.location.search);

    const code = urlParams.get("code");
    if (!code) {
      redirectToGithubOAuthUrl();
      return;
    }

    const installationId = urlParams.get("installation_id");
    console.log(`Working on installation ${installationId}`);

    validateCsrfState(urlParams);
    let auth = await exchangeCodeForToken(code);
    console.log("Got token back");
  });

  function redirectToGithubOAuthUrl() {
    let requestCsrfState = Math.random().toString(36).substring(2);
    sessionStorage.setItem(stateSessionKey, requestCsrfState);

    const url = new URL("https://github.com/login/oauth/authorize");
    url.searchParams.append("client_id", devClientId);
    const redirectUri = `${location.origin}/connections/github`;
    url.searchParams.append("redirect_uri", redirectUri);
    url.searchParams.append("state", requestCsrfState);

    console.log(
      `Redirecting to github oauth with redirect_uri ${redirectUri} and state ${requestCsrfState}`,
    );
    window.location.replace(url);
  }

  function validateCsrfState(urlParams: URLSearchParams) {
    const sessionCsrfState = sessionStorage.getItem(stateSessionKey);
    const csrfState = urlParams.get("state");
    if (sessionCsrfState !== csrfState) {
      console.warn(
        `CSRF state mismatch. Expected:'${sessionCsrfState}'', got:'${csrfState}''`,
      );
      // TODO
      throw new Error("Bad");
    }
  }
  async function exchangeCodeForToken(code: string): Promise<AuthResult> {
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
    return parseResponse(response);
  }
</script>

<Navbar />

<div></div>
