<script lang="ts">
  import { page } from "$app/state";
  import { headers, KosoError, parseResponse } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import { Alert } from "$lib/kosui/alert";
  import Button from "$lib/kosui/button/button.svelte";

  let auth = getAuthContext();
  let paramsPromise = $state(load());

  type ParsedParams = {
    responseType: string | null;
    clientId: string | null;
    /** We cannot trust the redirect URI until it's verified by the backend. */
    unvalidatedRedirectUri: string;
    scope: string | null;
    state: string | null;
    codeChallenge: string | null;
    codeChallengeMethod: string | null;
    resource: string | null;
    other: [string, string][];
  };

  type Params = Omit<ParsedParams, "unvalidatedRedirectUri"> & {
    clientName: string;
    validatedRedirectUri: string;
  };

  function parseParams(): ParsedParams {
    const urlParams = page.url.searchParams;
    function pop(name: string): string | null {
      const value = urlParams.get(name) || null;
      urlParams.delete(name);
      return value;
    }

    const responseType = pop("response_type");
    const clientId = pop("client_id");
    const unvalidatedRedirectUri = pop("redirect_uri");
    if (!unvalidatedRedirectUri) {
      toast.error(
        "Invalid parameters (missing redirect_uri). Close the page and try again.",
      );
      throw new Error("Empty redirectUri");
    }
    const scope = pop("scope");
    const state = pop("state");
    const codeChallenge = pop("code_challenge");
    const codeChallengeMethod = pop("code_challenge_method");
    const resource = pop("resource");

    // Collect any remaining parameters.
    const other = Array.from(urlParams.entries());

    return {
      responseType,
      clientId,
      unvalidatedRedirectUri,
      scope,
      state,
      codeChallenge,
      codeChallengeMethod,
      resource,
      other,
    };
  }

  async function load(): Promise<Params> {
    let params = parseParams();
    console.log(`Parsed authorization parameters: ${JSON.stringify(params)}`);

    const redirectUri = params.unvalidatedRedirectUri;
    const response = await fetch(`/oauth/authorization_details`, {
      method: "POST",
      headers: {
        ...headers(auth),
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        client_id: params.clientId,
        redirect_uri: redirectUri,
      }),
    });
    const details: { client_name: string } = await parseResponse(
      auth,
      response,
    );
    return {
      ...params,
      clientName: details.client_name,
      validatedRedirectUri: redirectUri,
    };
  }

  async function handleCancelClick() {
    const params = await paramsPromise;

    const redirectUri = newRedirectUri(params);
    redirectUri.searchParams.append("error", "access_denied");
    redirectUri.searchParams.append(
      "error_description",
      "Authorization cancelled by user.",
    );

    console.log(`Cancelled, redirecting back to client: ${redirectUri}`);
    window.location.replace(redirectUri);
  }

  async function handleAuthorizeClick() {
    const params = await paramsPromise;

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
          redirect_uri: params.validatedRedirectUri,
          resource: params.resource,
          other: params.other.length ? params.other : undefined,
        }),
      });
      const approval: { code: string } = await parseResponse(auth, response);

      const redirectUri = newSuccessRedirectUri(params, approval.code);
      console.info(`Approval request succeeded, redirecting: ${redirectUri}`);
      window.location.replace(redirectUri);
    } catch (e) {
      const redirectUri = newErrorRedirectUri(params, e);
      console.info(`Approval request failed, redirecting: ${redirectUri}`, e);
      window.location.replace(redirectUri);
    }
  }

  // https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2
  function newSuccessRedirectUri(params: Params, code: string) {
    const redirectUri = newRedirectUri(params);
    redirectUri.searchParams.append("code", code);
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
    const redirectUri = new URL(params.validatedRedirectUri);
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
      {#await paramsPromise}
        <div class="text-l">Loading...</div>
      {:then params}
        <div>Authorize access to "{params.clientName}"?</div>
        <div class="flex items-center gap-2">
          <Button onclick={handleAuthorizeClick}>Authorize</Button>
          <Button onclick={handleCancelClick}>Cancel</Button>
        </div>
      {:catch e}
        <div class="text-l">
          Something went wrong. Clear any state and try again.
        </div>
        <div class="text-red-500">{JSON.stringify(e)}</div>
      {/await}
    </div>
  </Alert>
</div>
