<script lang="ts">
  import { goto } from "$app/navigation";
  import { user } from "$lib/auth";
  import { pushRedirectOnUserNotAuthenticated } from "$lib/nav";

  $: if (!$user) {
    pushRedirectOnUserNotAuthenticated();
    goto("/");
  }
</script>

{#if $user}
  <slot></slot>
{:else}
  <div class="flex flex-col items-center justify-center rounded border p-4">
    <div class="text-l">Redirecting to login...</div>
  </div>
{/if}
