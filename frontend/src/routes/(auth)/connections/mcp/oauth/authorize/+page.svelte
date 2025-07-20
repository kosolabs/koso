<script lang="ts">
  import { headers, parseResponse } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import Button from "$lib/kosui/button/button.svelte";
  import { onMount } from "svelte";

  let auth = getAuthContext();

  type Params = {
    responseType: string;
    clientId: string;
    redirectUri: string;
    scope: string | null;
    state: string | null;
    codeChallenge: string | null;
    codeChallengeMethod: string | null;
    resource: string | null;
  };

  function parseParams(): Params {
    const urlParams = new URLSearchParams(window.location.search);
    const responseType = urlParams.get("response_type");
    if (responseType !== "code") {
      throw new Error(`Unsupported response type: ${responseType}`);
    }
    const clientId = urlParams.get("client_id");
    if (!clientId) {
      throw new Error("Empty client_id");
    }
    const redirectUri = urlParams.get("redirect_uri");
    if (!redirectUri) {
      throw new Error("Empty redirectUri");
    }
    const scope = urlParams.get("scope");
    const state = urlParams.get("state");
    const codeChallenge = urlParams.get("code_challenge");
    const codeChallengeMethod = urlParams.get("code_challenge_method");
    const resource = urlParams.get("resource");

    return {
      responseType,
      clientId,
      redirectUri,
      scope,
      state,
      codeChallenge,
      codeChallengeMethod,
      resource,
    };
  }
  async function handleAuthorizeClick() {
    let redirectUri = await authorize();
    console.log(`Redirecting back to client: ${redirectUri}`);
    window.location.assign(redirectUri);
  }

  async function authorize(): Promise<string> {
    const params = parseParams();
    const response = await fetch(`/oauth/approve`, {
      method: "POST",
      headers: {
        ...headers(auth),
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        client_id: params.clientId,
        scope: params.scope,
        code_challenge: params.codeChallenge,
        code_challenge_method: params.codeChallengeMethod,
        redirect_uri: params.redirectUri,
        resource: params.resource,
      }),
    });
    let approval: { code: string } = await parseResponse(auth, response);

    return `${params.redirectUri}?code=${encodeURIComponent(approval.code)}${params.state ? `&state=${encodeURIComponent(params.state)}` : ``}`;
  }

  onMount(() => {
    const params = parseParams();
    console.log(`Parased authorization parameters`, params);
  });
</script>

<Button onclick={handleAuthorizeClick}>Authorize</Button>
