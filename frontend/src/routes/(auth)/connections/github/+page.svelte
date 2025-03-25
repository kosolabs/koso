<script lang="ts">
  import { goto } from "$app/navigation";
  import { KosoError } from "$lib/api";
  import { Navbar } from "$lib/components/ui/navbar";
  import { toast } from "$lib/components/ui/sonner";
  import * as github from "$lib/github";
  import { onMount } from "svelte";

  onMount(async () => {
    // See https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#using-the-web-application-flow-to-generate-a-user-access-token
    const urlParams = new URLSearchParams(window.location.search);

    const state = parseAndValidateState(urlParams);
    if (!state) {
      await goto("/");
      return;
    }

    const code = urlParams.get("code");
    if (!code) {
      toast.info("Redirecting to Github for authorization");
      github.redirectToGitubOAuth(state);
      return;
    }

    console.log("Logging user in with Github");
    await authWithCode(code);

    console.log(
      `Connecting installation '${state.installationId}' and project '${state.projectId}'`,
    );
    await connectProject(state.projectId, state.installationId);
    toast.info("Project connected to Github!");

    await goto(`/projects/${state.projectId}`);
  });

  function parseAndValidateState(
    urlParams: URLSearchParams,
  ): github.State | null {
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

    const state = github.decodeState(stateParam);
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

    return {
      projectId: state.projectId,
      installationId: state.installationId,
      clientId: state.clientId,
      csrf: state.csrf,
    };
  }

  async function authWithCode(code: string): Promise<void> {
    try {
      await github.authWithCode(code);
    } catch (e) {
      if (e instanceof KosoError && e.hasReason("GITHUB_AUTH_REJECTED")) {
        toast.error("Failed to authenticate with Github. Please try again");
        await goto("/");
      }
      throw e;
    }
  }

  async function connectProject(
    projectId: string,
    installationId: string,
  ): Promise<void> {
    try {
      return github.connectProject(projectId, installationId);
    } catch (e) {
      if (e instanceof KosoError && e.hasReason("GITHUB_UNAUTHENTICATED")) {
        toast.error("Github authentication expired. Please try again");
        await goto("/");
      }
      throw e;
    }
  }
</script>

<Navbar />

<div class="p-4">Connecting...</div>
