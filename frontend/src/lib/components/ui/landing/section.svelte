<script lang="ts">
  import type { Snippet } from "svelte";

  export type SectionProps = {
    title: string;
    poster: string;
    video?: string;
    children: Snippet;
    appendix: Snippet;
  };
  let { poster, video, title, children, appendix }: SectionProps = $props();
</script>

<div class="flat-gradient px-4 pt-40 pb-20">
  <div
    class="mx-auto flex max-w-(--breakpoint-md) flex-col items-center gap-8 text-center"
  >
    <h2 class="text-2xl lg:text-4xl">{title}</h2>

    <p>{@render children()}</p>

    {#if video}
      <video
        class="placeholder flex w-full items-center justify-center rounded-lg border"
        autoplay
        loop
        muted
        playsinline
        src={video}
        {poster}
      ></video>
    {:else}
      <img
        src={poster}
        alt={title}
        class="placeholder flex w-full items-center justify-center rounded-lg border"
      />
    {/if}

    {@render appendix()}
  </div>
</div>

<style>
  p,
  div,
  h2,
  h3 {
    font-weight: 200;
    color: white;
  }

  .flat-gradient {
    background: radial-gradient(
      circle at bottom,
      hsl(195deg, 2%, 10%),
      hsl(195deg, 2%, 5%) 100%
    );
    box-shadow: 0 0 100px 10px hsl(195deg, 85%, 35%);
  }

  .placeholder {
    background: radial-gradient(
      circle at bottom left,
      hsl(195, 78%, 18%),
      hsl(200, 100%, 50%) 100%
    );
    box-shadow: 0 0 100px 20px hsl(250deg, 50%, 70%);
    border-width: 1px;
    border-color: white;
    border-radius: 1em;
  }
</style>
