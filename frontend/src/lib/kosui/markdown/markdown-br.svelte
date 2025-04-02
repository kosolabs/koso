<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    ...restProps
  }: MarkdownComponentProps<Tokens.Br> & HTMLAttributes<HTMLBRElement> =
    $props();
</script>

<br {...restProps} />
