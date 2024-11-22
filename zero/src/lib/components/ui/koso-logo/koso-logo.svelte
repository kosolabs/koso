<script lang="ts">
  import koso from "./koso.svg?raw";
  type Props = {
    class?: string;
  };
  const { class: classes }: Props = $props();
</script>

<div class={classes}>
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  {@html koso}
</div>
