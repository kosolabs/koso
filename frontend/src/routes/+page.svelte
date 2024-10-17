<script lang="ts">
  import { goto } from "$app/navigation";
  import kosoLogo from "$lib/assets/koso.svg";
  import { token, user } from "$lib/auth";
  import { lastVisitedProjectId, popRedirectOnLogin } from "$lib/nav";
  import { fetchProjects } from "$lib/projects";
  import Google from "./google.svelte";

  if ($user) {
    redirectOnLogin();
  }

  async function onsuccess(credential: string) {
    $token = credential;
    await redirectOnLogin();
  }

  async function redirectOnLogin() {
    // If the user tried to access a page while unauthenticated,
    // clear the redirect and go there.
    const redirect = popRedirectOnLogin();
    if (redirect) {
      console.debug(`Going to prior page: ${redirect}`);
      await goto(redirect);
      return;
    }

    // Go to the previously viewed project, if there is one.
    if ($lastVisitedProjectId) {
      console.debug(`Going to last visited project: ${$lastVisitedProjectId}`);
      await goto(`/projects/${$lastVisitedProjectId}`);
      return;
    }

    // If there's only 1 project, go to it.
    const projects = await fetchProjects($token);
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

{#if !$user}
  <div
    class="m-4 flex flex-col gap-8 rounded-xl border bg-card p-10 text-center shadow sm:mx-auto sm:w-96"
  >
    <img class="m-auto w-20" alt="Koso Logo" src={kosoLogo} />
    <h1 class="text-4xl text-primary">Koso</h1>
    <Google {onsuccess} />
  </div>
{/if}
