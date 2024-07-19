<script lang="ts">
  import type { User } from "$lib/auth";
  import { Avatar } from "flowbite-svelte";
  import { GoogleOAuthProvider } from "google-oauth-gsi";
  import { jwtDecode } from "jwt-decode";
  import { onMount } from "svelte";

  const CREDENTIAL_KEY = "credential";

  let token: string | null = sessionStorage.getItem(CREDENTIAL_KEY) || null;

  function decodeUser(token: string | null): User | null {
    if (token === null) {
      return null;
    }
    const user = jwtDecode(token) as User;
    if (user.exp * 1000 < Date.now()) {
      return null;
    }
    return user;
  }
  $: user = decodeUser(token);

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
            token = res.credential;
            sessionStorage.setItem(CREDENTIAL_KEY, res.credential);
          },
        })();
      },
    });
  });

  function logout() {
    token = null;
  }
</script>

<div class="m-auto w-96 rounded border bg-slate-100 p-10 text-center">
  <h1 class="mb-8 text-2xl">Sign in to Koso</h1>
  {#if user}
    <div
      class="flex items-center justify-center rounded-full border bg-slate-200 p-2"
    >
      <div><Avatar src={user.picture} /></div>
      <button class="pl-2" on:click={() => logout()}>Logout {user.name}</button>
    </div>
  {/if}
  <div id="google-login-button" hidden={!!token} />
</div>
