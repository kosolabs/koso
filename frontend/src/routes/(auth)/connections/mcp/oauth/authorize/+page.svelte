<script lang="ts">
  import { headers, KosoError, parseResponse } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { Alert } from "$lib/kosui/alert";
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
    const responseType = urlParams.get("response_type") || null;
    if (responseType !== "code") {
      throw new Error(`Unsupported response type: ${responseType}`);
    }
    const clientId = urlParams.get("client_id") || null;
    if (!clientId) {
      throw new Error("Empty client_id");
    }
    const redirectUri = urlParams.get("redirect_uri") || null;
    if (!redirectUri) {
      throw new Error("Empty redirectUri");
    }
    const scope = urlParams.get("scope") || null;
    const state = urlParams.get("state") || null;
    const codeChallenge = urlParams.get("code_challenge") || null;
    const codeChallengeMethod = urlParams.get("code_challenge_method") || null;
    const resource = urlParams.get("resource") || null;

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

  async function handleCancelClick() {
    const params = parseParams();
    const stateParam = params.state
      ? `&state=${encodeURIComponent(params.state)}`
      : ``;
    let redirectUri = `${params.redirectUri}?error=access_denied&error_description=${encodeURIComponent("Authorization cancelled by user.")}${stateParam}`;

    console.log(`Redirecting cancellation back to client: ${redirectUri}`);
    window.location.assign(redirectUri);
  }

  async function handleAuthorizeClick() {
    let redirectUri = await authorize();
    console.log(`Redirecting back to client: ${redirectUri}`);
    window.location.assign(redirectUri);
  }

  async function authorize(): Promise<string> {
    const params = parseParams();
    const stateParam = params.state
      ? `&state=${encodeURIComponent(params.state)}`
      : ``;

    try {
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
      // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2
      return `${params.redirectUri}?code=${encodeURIComponent(approval.code)}${stateParam}`;
    } catch (e) {
      console.error("Approval request failed: ", e);

      // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1
      const error =
        (e instanceof KosoError && e.details[0]?.reason) || "server_error";
      const error_description =
        (e instanceof KosoError && e.details[0]?.msg) ||
        "Something unexpected went wrong!";
      return `${params.redirectUri}?error=${encodeURIComponent(error)}&error_description=${encodeURIComponent(error_description)}${stateParam}`;
    }
  }

  onMount(() => {
    const params = parseParams();
    console.log(`Parsed authorization parameters`, params);
  });
</script>

<Navbar />

<div class="m-2">
  <Alert>
    <div class="flex flex-col items-center gap-2">
      <div>Click to authorize access to Koso.</div>
      <div class="flex items-center gap-2">
        <Button onclick={handleAuthorizeClick}>Authorize</Button>
        <Button onclick={handleCancelClick}>Cancel</Button>
      </div>
    </div>
  </Alert>
</div>
