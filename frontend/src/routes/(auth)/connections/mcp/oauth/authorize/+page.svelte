<script lang="ts">
  import { headers, KosoError, parseResponse } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import { Alert } from "$lib/kosui/alert";
  import Button from "$lib/kosui/button/button.svelte";
  import { onMount } from "svelte";

  let auth = getAuthContext();

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
    const redirectUri = await authorize();
    console.log(`Authorized, redirecting back to client: ${redirectUri}`);
    window.location.assign(redirectUri);
  }

  async function authorize(): Promise<URL> {
    const params = parseParams();
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

      const approval: { code: string } = await parseResponse(auth, response);
      // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2
      const redirectUri = newRedirectUri(params);
      redirectUri.searchParams.append("code", approval.code);
      return redirectUri;
    } catch (e) {
      console.error("Approval request failed: ", e);

      // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2.1
      const redirectUri = newRedirectUri(params);
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
      redirectUri.searchParams.append("error", error);
      redirectUri.searchParams.append("error", errorDescription);

      return redirectUri;
    }
  }

  function newRedirectUri(params: Params) {
    const redirectUri = new URL(params.redirectUri);
    if (params.state) {
      redirectUri.searchParams.append("state", params.state);
    }
    return redirectUri;
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
