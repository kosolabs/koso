<script lang="ts">
  import { logout, token, user } from "$lib/auth";
  import { Avatar } from "flowbite-svelte";
  import { GoogleOAuthProvider } from "google-oauth-gsi";
  import { onMount } from "svelte";

  onMount(() => {
    const googleProvider = new GoogleOAuthProvider({
      clientId:
        "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com",
      onScriptLoadSuccess: () => {
        googleProvider.useRenderButton({
          element: document.getElementById("google-login-button")!,
          useOneTap: true,
          width: 300,
          use_fedcm_for_prompt: true,
          onSuccess: (res) => {
            if (!res.credential) {
              console.error("Credential is missing", res);
              return;
            }
            $token = res.credential;
          },
        })();
      },
    });
  });
</script>

<div
  class="m-auto flex w-96 flex-col rounded border bg-slate-100 p-10 text-center"
>
  <h1 class="mb-8 text-2xl">Sign in to Koso</h1>
  {#if $user}
    <div class="m-auto">
      <div class="flex items-center rounded-full border bg-slate-200 p-2">
        <div><Avatar src={$user.picture} size="xs" /></div>
        <button class="pl-2" on:click={() => logout()}>
          Logout {$user.name}
        </button>
      </div>
    </div>
  {/if}
  <div id="google-login-button" hidden={!!$user} />
</div>
