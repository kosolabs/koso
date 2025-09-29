<script module lang="ts">
  import { goto } from "$app/navigation";
  import type { ResolvedPathname } from "$app/types";
  import { Link, mergeProps, type LinkProps } from "kosui";
  import type { Snippet } from "svelte";

  export type GotoProps = {
    href: ResolvedPathname;
    children: Snippet;
  } & Omit<LinkProps, "href">;
</script>

<script lang="ts">
  let { children, href, el = $bindable(), ...props }: GotoProps = $props();
</script>

<Link
  bind:el
  {...mergeProps(props, {
    href,
    onclick: (event: MouseEvent) => {
      event.stopPropagation();
      event.preventDefault();
      // ResolvedPathName not yet supported.
      // eslint-disable-next-line svelte/no-navigation-without-resolve
      goto(href);
    },
  })}
>
  {@render children?.()}
</Link>
