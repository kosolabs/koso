<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/auth.svelte";
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import { nav } from "$lib/nav.svelte";
  import { fetchProjects } from "$lib/projects";
  import Google from "./google.svelte";

  if (auth.ok()) {
    redirectOnLogin();
  }

  async function onsuccess(credential: string) {
    auth.token = credential;
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
    if (projects.length == 1) {
      const onlyProjectId = projects[0].project_id;
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
  <div
    class="m-4 flex flex-col gap-8 rounded-xl border bg-card p-10 text-center shadow sm:mx-auto sm:w-96"
  >
    <KosoLogo class="m-auto w-20" />
    <h1 class="text-4xl text-primary">Koso</h1>
    <Google {onsuccess} />
  </div>
{/if}
