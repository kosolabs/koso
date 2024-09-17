<script lang="ts">
  export let title: string;
  export let description: string = "";
</script>

<div
  class="rounded border bg-primary-foreground px-1 text-xs max-sm:hidden"
  title={description}
>
  {title}
</div>
