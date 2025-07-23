<script lang="ts">
  import { headers, KosoError, parseResponse } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import { Alert } from "$lib/kosui/alert";
  import Button from "$lib/kosui/button/button.svelte";

  let auth = getAuthContext();
  let details = $state(load());

  type Params = {
    responseType: string | null;
    clientId: string | null;
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
    const clientId = urlParams.get("client_id") || null;
    const redirectUri = urlParams.get("redirect_uri") || null;
    if (!redirectUri) {
      toast.error(
        "Invalid parameters (missing redirect_uri). Close the page and try again.",
      );
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

  type AuthorizationDetails = {
    client_name?: string;
    error?: string;
  };

  async function load(): Promise<AuthorizationDetails> {
    let params = parseParams();
    console.log(`Parsed authorization parameters: ${JSON.stringify(params)}`);

    try {
      const response = await fetch(`/oauth/authorization_details`, {
        method: "POST",
        headers: {
          ...headers(auth),
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          client_id: params.clientId,
          redirect_uri: params.redirectUri,
        }),
      });
      return await parseResponse(auth, response);
    } catch (e) {
      console.log("Failed to fetch authorization details", e);
      return { error: JSON.stringify(e) };
    }
  }

  async function handleCancelClick() {
    const params = parseParams();
    const redirectUri = newRedirectUri(params);
    redirectUri.searchParams.append("error", "access_denied");
    redirectUri.searchParams.append(
      "error_description",
      "Authorization cancelled by user.",
    );

    console.log(`Cancelled, redirecting back to client: ${redirectUri}`);
    window.location.assign(redirectUri);
  }

  async function handleAuthorizeClick() {
    const redirectUri = await approve();
    console.log(`Authorized, redirecting back to client: ${redirectUri}`);
    window.location.assign(redirectUri);
  }

  async function approve(): Promise<URL> {
    const params = parseParams();
    let approval: { code: string };
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
      approval = await parseResponse(auth, response);
    } catch (e) {
      console.error("Approval request failed: ", e);
      return newErrorRedirectUri(params, e);
    }

    // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2
    const redirectUri = newRedirectUri(params);
    redirectUri.searchParams.append("code", approval.code);
    return redirectUri;
  }

  // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1
  function newErrorRedirectUri(params: Params, e: unknown) {
    let error;
    let errorDescription;
    if (e instanceof KosoError) {
      const detail = e.details[0];
      if (detail) {
        error = detail.reason;
        errorDescription = detail.msg;
      } else {
        if (e.status === 400) {
          error = "invalid_request";
          errorDescription = "Invalid approval request.";
        } else {
          error = "server_error";
          errorDescription = "Something unexpected went wrong!";
        }
      }
    } else {
      error = "server_error";
      errorDescription = "Something unexpected went wrong!";
    }

    const redirectUri = newRedirectUri(params);
    redirectUri.searchParams.append("error", error);
    redirectUri.searchParams.append("error_description", errorDescription);

    return redirectUri;
  }

  function newRedirectUri(params: Params) {
    const redirectUri = new URL(params.redirectUri);
    if (params.state) {
      redirectUri.searchParams.append("state", params.state);
    }
    return redirectUri;
  }
</script>

<Navbar />

<div class="m-2">
  <Alert>
    <div class="flex flex-col items-center gap-2">
      {#await details}
        <div class="text-l">Loading...</div>
      {:then details}
        {#if details.client_name}
          <div>Authorize access to "{details.client_name}"?</div>
          <div class="flex items-center gap-2">
            <Button onclick={handleAuthorizeClick}>Authorize</Button>
            <Button onclick={handleCancelClick}>Cancel</Button>
          </div>
        {:else}
          <div class="text-l">
            Something went wrong. Clear any state and try again.
          </div>
          <div class="text-red-500">{details.error}</div>
        {/if}
      {/await}
    </div>
  </Alert>
</div>
