<script lang="ts">
  import { logout } from "$lib/auth";
  import { onMount } from "svelte";
  import { Alert, Button, A } from "flowbite-svelte";
  import { goto } from "$app/navigation";
  import { createProject, fetchProjects } from "$lib/projects";

  onMount(() => {
    logout();
    goto("/");
  });
</script>

<div class="flex flex-col rounded border p-4">Logging out...</div>
