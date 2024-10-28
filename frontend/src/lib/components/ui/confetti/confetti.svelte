<script module lang="ts">
  class ConfettiState {
    rects: DOMRect[] = $state([]);

    add(rect: DOMRect) {
      this.rects.push(rect);
      setTimeout(() => this.rects.shift(), 2000);
    }
  }
  export const confetti = new ConfettiState();
</script>

<script lang="ts">
  import Confetti from "svelte-confetti";
</script>

{#each confetti.rects as rect}
  {@const left = rect.x + rect.width / 2}
  {@const top = rect.y + rect.height / 2}
  <div class="fixed" style="left:{left}px;top:{top}px;">
    <Confetti />
  </div>
{/each}
