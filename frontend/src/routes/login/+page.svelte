<script lang="ts">
  import { GoogleOAuthProvider } from "google-oauth-gsi";
  import { onMount } from "svelte";

  let credential: string | null = null;

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
            credential = res.credential;
            console.log(res);
          },
        })();
      },
    });
  });
</script>

<div class="m-auto w-96 rounded border bg-slate-100 p-10 text-center">
  <h1 class="h1 mb-8 text-2xl">Sign in to Koso</h1>
  {#if credential}
    {credential}
  {:else}
    <div id="google-login-button" />
  {/if}
</div>
