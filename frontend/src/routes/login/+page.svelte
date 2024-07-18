<script lang="ts">
  import { GoogleOAuthProvider, googleLogout } from "google-oauth-gsi";
  import { onMount } from "svelte";

  export const googleProvider = new GoogleOAuthProvider({
    clientId:
      "560654064095-kicdvg13cb48mf6fh765autv6s3nhp23.apps.googleusercontent.com",
    onScriptLoadError: () => console.log("onScriptLoadError"),
    onScriptLoadSuccess: () => {},
  });

  onMount(() => {
    const oneTap = googleProvider.useGoogleOneTapLogin({
      cancel_on_tap_outside: true,
      onSuccess: (res) => console.log("Logged in with google one tap", res),
    });
    oneTap();
  });

  const login = googleProvider.useGoogleLogin({
    flow: "auth-code",
    onSuccess: (res) => console.log("Logged in with google", res),
    onError: (err) => console.error("Failed to login with google", err),
  });
  // Call login() on button click
</script>

<button
  class="absolute left-1 z-50 rounded p-1 opacity-50 outline hover:opacity-100"
  on:click={() => login()}
>
  Login
</button>
