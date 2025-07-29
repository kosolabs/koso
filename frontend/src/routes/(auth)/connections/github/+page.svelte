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
      const redirectUri = `${location.origin}/connections/github`;
      github.redirectToGitubOAuth(state, redirectUri);
      return;
    }

    console.log(
      `Connecting installation '${state.installationId}' and project '${state.projectId}'`,
    );
    await connectProject(state, code);
    toast.info("Project connected to Github!");

    await goto(state.redirectUrl, { replaceState: true });
  });

  function parseAndValidateState(
    urlParams: URLSearchParams,
  ): github.ConnectProjectState | null {
    const stateParam = urlParams.get("state");
    if (!stateParam) {
      // TODO: Implement an installation/project picker.
      // The user installed the app from Github which redirected here without a state parameter.
      console.log("No state parameter present");
      toast.warning(
        "App installed. Connect to Koso by clicking the 'Connect to Github' button on your project page",
      );
      return null;
    }
    if (!github.validateStateForCsrf(stateParam)) {
      toast.error(
        "Something went wrong connecting to Github. Please try again",
      );
      return null;
    }

    const state = github.decodeState<github.ConnectProjectState>(stateParam);
    console.log("Decoded state", state);

    // Add the installation ID passed as a query parameter to the state.
    const installationIdParam = urlParams.get("installation_id");
    if (!state.installationId && installationIdParam) {
      state.installationId = installationIdParam;
    }
    if (installationIdParam && state.installationId !== installationIdParam) {
      console.log(
        `Installation param (${installationIdParam}) doesn't match state (${state.installationId})`,
      );
      toast.error(
        "Something went wrong, installation mismatch. Connect via the 'Connect' button on your project page",
      );
      return null;
    }

    if (!state.installationId) {
      // TODO: Implement an installation picker.
      // The navigated directly to the connections page
      console.log("No installation ID present");
      toast.warning(
        "No installation selected. Connect via the 'Connect to Github' button on your project page",
      );
      return null;
    }
    if (!state.projectId) {
      // TODO: Implement a project selector that redirects to the project.
      // The user installed the app via the market place rather than our share button.
      console.log("No project ID selected");
      toast.warning(
        "No project selected. Connect via the 'Connect to Github' button on your project page",
      );
      return null;
    }
    if (!state.clientId || !state.csrf) {
      console.warn("No client id or csrf present in state", state);
      toast.error(
        "Something went wrong, invalid state. Connect via the 'Connect' button on your project page",
      );
      return null;
    }
    if (!state.redirectUrl) {
      console.warn("No redirectUrl present in state", state);
      toast.error(
        "Something went wrong, invalid state. Connect via the 'Connect' button on your project page",
      );
      return null;
    }

    return {
      projectId: state.projectId,
      installationId: state.installationId,
      clientId: state.clientId,
      csrf: state.csrf,
      redirectUrl: state.redirectUrl,
    };
  }

  async function connectProject(
    state: github.ConnectProjectState,
    code: string,
  ): Promise<void> {
    try {
      return github.connectProject(
        auth,
        state.projectId,
        state.installationId,
        code,
      );
    } catch (e) {
      if (e instanceof KosoError) {
        if (e.hasReason("GITHUB_UNAUTHENTICATED")) {
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
