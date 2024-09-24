<script lang="ts">
  import { mount, unmount, type Snippet } from "svelte";
  import PortalConsumer from "./portal-consumer.svelte";

  type Props = { children?: Snippet; target?: HTMLElement };
  const { children, target = document.body }: Props = $props();

  $effect(() => {
    const instance = mount(PortalConsumer, { target, props: { children } });
    return () => unmount(instance);
  });
</script>
