<script lang="ts">
  export let title: string = "";
</script>

<div class="whitespace-nowrap text-sm max-md:hidden" {title}>
  <slot />
</div>
