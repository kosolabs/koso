<script lang="ts">
  import type { Tokens } from "marked";
  import type { HTMLAttributes } from "svelte/elements";
  import type { MarkdownComponentProps } from ".";

  let {
    ...restProps
  }: MarkdownComponentProps<Tokens.Hr> & HTMLAttributes<HTMLHRElement> =
    $props();
</script>

<hr {...restProps} />
