<script lang="ts">
  import { goto } from "$app/navigation";
  import kosoLogo from "$lib/assets/koso.svg";
  import { logout, token, user } from "$lib/auth";
  import { fetchProjects } from "$lib/projects";
  import { lastVisitedProjectId, popRedirectOnLogin } from "$lib/nav";
  import { Alert, Avatar, Button } from "flowbite-svelte";
  import { GoogleOAuthProvider } from "google-oauth-gsi";
  import { onMount } from "svelte";
  import Google from "./google.svelte";

  let googleLogin: () => void;
  let errorMessage: string | null = null;

  function login() {
    errorMessage = null;
    googleLogin();
  }

  async function redirectOnLogin() {
    // If the user tried to access a page while unauthenticated,
    // clear the redirect and go there.
    const redirect = popRedirectOnLogin();
    if (redirect) {
      console.log(`Going to prior page: ${redirect}`);
      await goto(redirect);
      return;
    }

    // Go to the previously viewed project, if there is one.
    if ($lastVisitedProjectId) {
      console.log(`Going to last visited project: ${$lastVisitedProjectId}`);
      await goto(`/projects/${$lastVisitedProjectId}`);
      return;
    }

    // If there's only 1 project, go to it.
    const projects = await fetchProjects($token);
    if (projects.length == 1) {
      const onlyProjectId = projects[0].project_id;
      console.log(`Going to singular project: ${onlyProjectId}`);
      await goto(`/projects/${onlyProjectId}`);
      return;
    }

    // If there's no better choice, go to the projects page.
    console.log("Going to /projects");
    await goto(`/projects`);
  }

  onMount(() => {
    const googleProvider = new GoogleOAuthProvider({
      clientId:
        "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com",
      onScriptLoadSuccess: () => {
        googleLogin = googleProvider.useGoogleOneTapLogin({
          cancel_on_tap_outside: true,
          use_fedcm_for_prompt: true,
          onSuccess: async (oneTapResponse) => {
            if (!oneTapResponse.credential) {
              console.error("Credential is missing", oneTapResponse);
              return;
            }
            const loginResponse = await fetch("/api/auth/login", {
              method: "POST",
              headers: {
                Authorization: `Bearer ${oneTapResponse.credential}`,
              },
            });
            if (loginResponse.ok) {
              $token = oneTapResponse.credential!;
              await redirectOnLogin();
            } else {
              errorMessage = `Failed to login: ${loginResponse.statusText} (${loginResponse.status})`;
            }
          },
        });
      },
    });
  });
</script>

<div
  class="m-auto flex flex-col rounded border bg-slate-100 p-10 text-center lg:w-96"
>
  <img class="m-auto mb-8 w-20" alt="Koso Logo" src={kosoLogo} />
  <h1 class="mb-8 text-4xl text-teal-800">Koso</h1>
  {#if $user}
    <div class="m-auto my-2">
      <div class="flex items-center rounded-full border bg-slate-200 p-2">
        <div><Avatar src={$user.picture} size="xs" /></div>
        <button class="pl-2" on:click={() => logout()}>
          Logout {$user.name}
        </button>
      </div>
    </div>

    <h1 class="mt-8 text-xl">Projects</h1>

    <div class="m-auto my-2">
      <Button on:click={() => goto("/projects/koso-staging")}>
        koso-staging
      </Button>
    </div>
  {:else}
    <Google on:click={login} />
  {/if}
  {#if errorMessage}
    <Alert class="mt-8" border>{errorMessage}</Alert>
  {/if}
</div>
