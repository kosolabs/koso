<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/auth.svelte";
  import { nav } from "$lib/nav.svelte";
  import { fetchProjects } from "$lib/projects";
  import Landing from "./landing.svelte";

  if (auth.ok()) {
    redirectOnLogin();
  }

  async function onsuccess() {
    await redirectOnLogin();
  }

  async function redirectOnLogin() {
    // If the user tried to access a page while unauthenticated,
    // clear the redirect and go there.
    const redirect = nav.popRedirectOnLogin();
    if (redirect) {
      console.debug(`Going to prior page: ${redirect}`);
      await goto(redirect);
      return;
    }

    // Go to the previously viewed project, if there is one.
    if (nav.lastVisitedProjectId) {
      console.debug(
        `Going to last visited project: ${nav.lastVisitedProjectId}`,
      );
      await goto(`/projects/${nav.lastVisitedProjectId}`);
      return;
    }

    // If there's only 1 project, go to it.
    const projects = await fetchProjects();
    if (projects.length === 1) {
      const onlyProjectId = projects[0].projectId;
      console.debug(`Going to singular project: ${onlyProjectId}`);
      await goto(`/projects/${onlyProjectId}`);
      return;
    }

    // If there's no better choice, go to the projects page.
    console.debug("Going to /projects");
    await goto(`/projects`);
  }
</script>

{#if !auth.ok()}
  <Landing {onsuccess} />
{/if}
