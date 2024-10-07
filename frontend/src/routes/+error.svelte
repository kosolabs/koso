<script>
  import { page } from "$app/stores";
  let message = $page.error?.message;
</script>

<div class="text-center">
  <div
    class="m-4 inline-block max-w-lg rounded-lg bg-red-400 bg-opacity-40 p-6 px-12 text-center text-red-800 dark:bg-red-500 dark:bg-opacity-20 dark:text-red-400"
  >
    <p class="text-xl font-semibold">Error {$page.status}</p>
    <p>{message ?? "Something went wrong!"}</p>
  </div>
</div>
