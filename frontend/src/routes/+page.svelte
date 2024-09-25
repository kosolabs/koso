<script lang="ts">
  import { goto } from "$app/navigation";
  import kosoLogo from "$lib/assets/koso.svg";
  import { token, user } from "$lib/auth";
  import { Alert } from "$lib/components/ui/alert";
  import { lastVisitedProjectId, popRedirectOnLogin } from "$lib/nav";
  import { fetchProjects } from "$lib/projects";
  import { GoogleOAuthProvider } from "google-oauth-gsi";
  import { onMount } from "svelte";
  import Google from "./google.svelte";

  if ($user) {
    redirectOnLogin();
  }

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

  onMount(() => {
    if ($user) {
      return;
    }

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

{#if !$user}
  <div
    class="mx-auto my-4 flex flex-col gap-8 rounded-xl border bg-card p-10 text-center shadow lg:w-96"
  >
    <img class="m-auto w-20" alt="Koso Logo" src={kosoLogo} />
    <h1 class="text-4xl text-primary">Koso</h1>
    <Google on:click={login} />
    {#if errorMessage}
      <Alert variant="destructive">{errorMessage}</Alert>
    {/if}
  </div>
{/if}
