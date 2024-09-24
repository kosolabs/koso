<script lang="ts">
  import { mount, unmount, type Snippet } from "svelte";
  import Proxy from "./proxy.svelte";

  type Props = { children?: Snippet; target?: HTMLElement };
  const { children, target = document.body }: Props = $props();

  $effect(() => {
    const instance = mount(Proxy, { target, props: { children } });
    return () => unmount(instance);
  });
</script>
