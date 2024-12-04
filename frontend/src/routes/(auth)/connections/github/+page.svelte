<script lang="ts">
  import Navbar from "$lib/navbar.svelte";
  import { KosoError } from "$lib/api";
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { goto } from "$app/navigation";
  import * as github from "$lib/github";

  onMount(async () => {
    // See https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-a-user-access-token-for-a-github-app#using-the-web-application-flow-to-generate-a-user-access-token
    const urlParams = new URLSearchParams(window.location.search);

    let state = urlParams.get("state");
    if (state && !github.validateCsrfState(state)) {
      toast.error(
        "Something went wrong connecting to Github. Please try again",
      );
      await goto("/projects");
      return;
    }

    const code = urlParams.get("code");
    if (!code) {
      toast.warning(
        "No Github auth code is present.  Connect via the 'Connect' button on your project page",
      );
      await goto("/");
      return;
    }

    console.log("Logging user in with Github");
    await authWithCode(code);

    const installationId = urlParams.get("installation_id");
    const projectId = state && github.decodeCsrfStateAsProjectId(state);
    if (!installationId) {
      // TODO: Implement an installation picker.
      // The navigated directly to the connections page
      toast.warning(
        "No installation selected. Connect via the 'Connect' button on your project page",
      );
      await goto("/");
      return;
    }
    if (!projectId) {
      // TODO: Implement a project selector that redirects to the project.
      // The user installed the app via the market place rather than our share button.
      console.log("No project ID selected");
      toast.warning(
        "No project selected. Connect via the 'Connect' button on your project page",
      );

      await goto("/");
      return;
    }

    console.log(
      `Connecting installation '${installationId}' and project '${projectId}'`,
    );
    await connectProject(projectId, installationId);
    toast.info("Project connected to Github!");

    await goto(`/projects/${projectId}`);
  });

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

<div></div>
