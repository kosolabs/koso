<script module lang="ts">
  class ConfettiState {
    rects: { el: DOMRect; id: number }[] = $state([]);
    id = 1;

    add(rect: DOMRect) {
      this.rects.push({ el: rect, id: this.id++ });
      setTimeout(() => this.rects.shift(), 2000);
    }
  }
  export const confetti = new ConfettiState();
</script>

<script lang="ts">
  import Confetti from "svelte-confetti";
</script>

{#each confetti.rects as rect (rect.id)}
  {@const left = rect.el.x + rect.el.width / 2}
  {@const top = rect.el.y + rect.el.height / 2}
  <div class="fixed" style="left:{left}px;top:{top}px;">
    <Confetti />
  </div>
{/each}
