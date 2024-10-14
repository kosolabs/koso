<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  type Props = {
    children?: Snippet;
    progress: number;
    size?: string;
    thickness?: string;
    trackColor?: string;
    color?: string;
    class?: string;
  };

  const {
    children,
    progress,
    size = "22px",
    thickness = "2px",
    trackColor = "color-mix(in srgb, currentColor 10%, transparent)",
    color = "currentColor",
    class: classes,
  }: Props = $props();
</script>

<div class={cn("p-[1px]", classes)} aria-label="circular-progress">
  <div
    class="circular-progress"
    style="
      --cp-progress:{progress};
      --cp-size:{size};
      --cp-thickness:{thickness};
      --cp-track-color:{trackColor};
      --cp-fill-color:{color};"
  >
    <div class="slot">
      {@render children?.()}
    </div>
    <svg class="progress-bar">
      <circle class="track" />
      <circle class="progress" />
    </svg>
  </div>
</div>

<style>
  .circular-progress {
    --_cp-radius: calc(var(--cp-size) / 2 - var(--cp-thickness) / 2);
    --_cp-length: calc(2 * 3.1415926535 * var(--_cp-radius));
    width: var(--cp-size);
    height: var(--cp-size);
    position: relative;
  }

  .progress-bar {
    width: var(--cp-size);
    height: var(--cp-size);
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    fill: transparent;
  }

  .slot {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: calc(var(--cp-size) / 3);
  }

  .track {
    cx: 50%;
    cy: 50%;
    r: var(--_cp-radius);
    stroke: var(--cp-track-color);
    stroke-width: var(--cp-thickness);
  }

  .progress {
    cx: 50%;
    cy: 50%;
    r: var(--_cp-radius);
    stroke: var(--cp-fill-color);
    stroke-width: var(--cp-thickness);
    stroke-linecap: round;
    stroke-dasharray: var(--_cp-length);
    stroke-dashoffset: calc((1 - var(--cp-progress)) * var(--_cp-length));
    transform-origin: center;
    transform: rotate(-90deg);
  }
</style>
