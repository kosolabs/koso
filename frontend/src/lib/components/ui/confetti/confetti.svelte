<script module lang="ts">
  const DURATION_MS = 2000;
  class ConfettiState {
    rects = $state(new SvelteMap<number, DOMRect>());
    id = 1;

    add(rect: DOMRect) {
      const id = this.id++;
      this.rects.set(id, rect);
      setTimeout(() => this.rects.delete(id), DURATION_MS + 25);
    }
  }
  export const confetti = new ConfettiState();
</script>

<script lang="ts">
  import Confetti from "svelte-confetti";
  import { SvelteMap } from "svelte/reactivity";
</script>

{#each confetti.rects as [id, rect] (id)}
  {@const left = rect.x + rect.width / 2}
  {@const top = rect.y + rect.height / 2}
  <div class="fixed" style="left:{left}px;top:{top}px;">
    <Confetti duration={DURATION_MS} x={[-0.65, 0.65]} y={[0.1, 1]} />
  </div>
{/each}
