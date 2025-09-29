<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { getAuthContext } from "$lib/auth.svelte";
  import { nav } from "$lib/nav.svelte";
  import { fetchProjects } from "$lib/projects";
  import Landing from "./landing.svelte";

  const auth = getAuthContext();
  let authOk: boolean = $derived(auth.ok());

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
      // Guaranteed to be of the same origin by `nav`
      // eslint-disable-next-line svelte/no-navigation-without-resolve
      await goto(redirect);
      return;
    }

    // Go to the previously viewed project, if there is one.
    if (nav.lastVisitedProjectId) {
      console.debug(
        `Going to last visited project: ${nav.lastVisitedProjectId}`,
      );
      await goto(resolve(`/projects/${nav.lastVisitedProjectId}`));
      return;
    }

    // If there's only 1 project, go to it.
    const projects = await fetchProjects(auth);
    if (projects.length == 1) {
      const onlyProjectId = projects[0].projectId;
      console.debug(`Going to singular project: ${onlyProjectId}`);
      await goto(resolve(`/projects/${onlyProjectId}`));
      return;
    }

    // If there's no better choice, go to the projects page.
    console.debug("Going to /projects");
    await goto(resolve(`/projects`));
  }
</script>

{#if !authOk}
  <Landing {onsuccess} />
{/if}
