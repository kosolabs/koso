<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { KosoError } from "$lib/api";
  import { getAuthContext } from "$lib/auth.svelte";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import * as github from "$lib/github";
  import { onMount } from "svelte";

  const auth = getAuthContext();

  onMount(async () => {
    // See https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#using-the-web-application-flow-to-generate-a-user-access-token
    const urlParams = page.url.searchParams;

    const state = parseAndValidateState(urlParams);
    if (!state) {
      await goto("/");
      return;
    }

    // When we initially land on this page, the code parameter will be absent.
    // Send the user to Github to authenticate. When that's done, Github
    // will redirect back here with the code parameter set.
    const code = urlParams.get("code");
    if (!code) {
      toast.info("Redirecting to Github for authorization");
      const redirectUri = `${location.origin}/connections/github/user`;
      github.redirectToGitubOAuth(state, redirectUri);
      return;
    }

    console.log(`Connecting user`);
    await connectUser(state, code);
    toast.success("User connected to Github!");

    await goto(state.redirectUrl, { replaceState: true });
  });

  function parseAndValidateState(
    urlParams: URLSearchParams,
  ): github.ConnectUserState | null {
    const stateParam = urlParams.get("state");
    if (!stateParam) {
      // The user must have navigated here manually without setting a state parameter.
      console.log("No state parameter present");
      toast.error("Something went wrong, missing state. Please try again");
      return null;
    }
    if (!github.validateStateForCsrf(stateParam)) {
      toast.error(
        "Something went wrong connecting to Github. Please try again",
      );
      return null;
    }

    const state = github.decodeState<github.ConnectUserState>(stateParam);
    console.log("Decoded state", state);

    if (!state.clientId || !state.csrf) {
      console.warn("No client id or csrf present in state", state);
      toast.error("Something went wrong, invalid state. Please try again");
      return null;
    }
    if (!state.redirectUrl) {
      console.warn("No redirectUrl present in state", state);
      toast.error("Something went wrong, invalid state. Please try again");
      return null;
    }

    return {
      clientId: state.clientId,
      csrf: state.csrf,
      redirectUrl: state.redirectUrl,
    };
  }

  async function connectUser(
    state: github.ConnectUserState,
    code: string,
  ): Promise<void> {
    try {
      return github.connectUser(auth, code);
    } catch (e) {
      if (e instanceof KosoError) {
        if (e instanceof KosoError && e.hasReason("GITHUB_UNAUTHENTICATED")) {
          toast.error("Github authentication expired. Please try again");
          await goto(state.redirectUrl);
        } else if (e.hasReason("GITHUB_AUTH_REJECTED")) {
          toast.error("Failed to authenticate with Github. Please try again");
          await goto(state.redirectUrl);
        }
      }
      throw e;
    }
  }
</script>

<Navbar />

<div class="p-4">Connecting...</div>
