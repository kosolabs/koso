<script lang="ts">
  import { KosoLogo } from "$lib/components/ui/koso-logo";
  import editDemo from "$lib/components/ui/landing/edit-demo.png";
  import collabDemo from "$lib/components/ui/landing/collab-demo.png";
  import linkDemo from "$lib/components/ui/landing/link-demo.png";
  import githubDemo from "$lib/components/ui/landing/github-demo.png";
  import Google from "./google.svelte";

  type Props = {
    onsuccess: () => void;
  };
  const { onsuccess }: Props = $props();

  let heroTextEl: HTMLDivElement | undefined = $state();

  function updateHeroTextDisplay() {
    if (!heroTextEl) return;
    const t = Math.max(Math.min((window.scrollY - 50) / 250, 1), 0);
    heroTextEl.style.opacity = `${1 - t}`;
    heroTextEl.style.scale = `${1 - 0.2 * t}`;
  }

  $effect(() => {
    updateHeroTextDisplay();
  });

  $effect(() => {
    window.addEventListener("scroll", updateHeroTextDisplay);
    return () => window.removeEventListener("scroll", updateHeroTextDisplay);
  });
</script>

<div class="colorful-gradient flex flex-col items-center">
  <div class="hero-text text-center" bind:this={heroTextEl}>
    <h1 class="pb-5 pt-20 text-6xl lg:pt-40">
      <span class="inline-block">Plan Less.</span>
      <span class="inline-block">Ship More.</span>
    </h1>
    <div>
      <p>Reduce project planning time to zero.</p>
      <p class="text-xs">(or as close to zero as is possible)</p>
    </div>
  </div>

  <div
    class="glass mx-4 my-20 flex max-w-screen-md flex-col items-center gap-8 p-10 text-center"
  >
    <KosoLogo class="m-auto w-20" />
    <h1 class="text-4xl">Koso</h1>
    <p>
      We're not quite ready for prime time just yet, but if you have a
      pre-launch invite, login with Google.
    </p>
    <Google {onsuccess} />
  </div>
</div>

<div class="flat-gradient px-4 pb-20 pt-40">
  <div
    class="mx-auto flex max-w-screen-md flex-col items-center gap-8 text-center"
  >
    <h2 class="text-2xl lg:text-4xl">Plan at the Speed of Thought</h2>
    <p>Create and edit tasks as if you were typing in a doc.</p>
    <div
      class="placeholder flex aspect-video w-full items-center justify-center rounded-lg border"
    >
      <img src={editDemo} alt="Create and edit tasks" />
    </div>
    <div>
      <h3 class="pb-4 text-xl lg:text-2xl">Quick Edits</h3>
      <p>
        Use keyboard shortcuts to create, edit and change the status of tasks,
        move tasks up and down, and change their indentation.
      </p>
    </div>
    <div>
      <h3 class="pb-4 text-xl lg:text-2xl">Infinite Undo</h3>
      <p>
        Don't worry about saving or applying your changes. Undo your mistakes.
      </p>
    </div>
  </div>
</div>

<div class="flat-gradient px-4 pb-20 pt-40">
  <div
    class="mx-auto flex max-w-screen-md flex-col items-center gap-8 text-center"
  >
    <h2 class="text-2xl lg:text-4xl">Collaborative</h2>
    <p>
      Multiple people can organize tasks at the same time. Ask the gizmo expert
      to break down the tasks for gizmo's whatchamacallit feature.
    </p>
    <div
      class="placeholder flex aspect-video w-full items-center justify-center rounded-lg border"
    >
      <img src={collabDemo} alt="Multiple people collaborating on tasks" />
    </div>
  </div>
</div>

<div class="flat-gradient px-4 pb-20 pt-40">
  <div
    class="mx-auto flex max-w-screen-md flex-col items-center gap-8 text-center"
  >
    <h2 class="text-2xl lg:text-4xl">Simplified Dependency Management</h2>
    <p>
      Tasks are organized as a graph. Drag and drop tasks to link a task as the
      dependency of another. Or link by searching.
    </p>
    <div
      class="placeholder flex aspect-video w-full items-center justify-center rounded-lg border"
    >
      <img src={linkDemo} alt="Link tasks by searching" />
    </div>
  </div>
</div>

<div class="flat-gradient px-4 pb-20 pt-40">
  <div
    class="mx-auto flex max-w-screen-md flex-col items-center gap-8 text-center"
  >
    <h2 class="text-2xl lg:text-4xl">Extensible Intelligent Automation</h2>
    <p>
      Tasks can be linked to external dependencies, like a GitHub PR for
      example. Externally linked tasks are tracked and automatically resolved.
      External tasks are managed through Koso plugins. Our list of plugins is
      constantly growing.
    </p>
    <div
      class="placeholder flex aspect-video w-full items-center justify-center rounded-lg border"
    >
      <img src={githubDemo} alt="External Github PR task" />
    </div>
  </div>
</div>

<div class="flat-gradient px-4 pb-20 pt-40">
  <div
    class="mx-auto flex max-w-screen-md flex-col items-center gap-8 text-center"
  >
    <h2 class="text-2xl lg:text-4xl">Self Organizing Zero Inbox</h2>
    <p>
      Only see tasks that are actionable. If an unactionable task appears in
      your inbox, add a blocker by linking the dependent task. Koso's task
      juggler will notify you when your task becomes actionable.
    </p>
    <div
      class="placeholder flex aspect-video w-full items-center justify-center rounded-lg border"
    >
      Coming soon!
    </div>
  </div>
</div>

<style>
  p,
  div,
  h1,
  h2,
  h3 {
    font-weight: 200;
    color: white;
  }

  .colorful-gradient {
    background: radial-gradient(
      circle at bottom,
      hsl(195deg, 85%, 35%),
      hsl(250deg 100% 5%) 100%
    );
    box-shadow: 0 0 100px 10px hsl(195deg, 85%, 35%);
  }

  .flat-gradient {
    background: radial-gradient(
      circle at bottom,
      hsl(195deg, 2%, 10%),
      hsl(195deg, 2%, 5%) 100%
    );
    box-shadow: 0 0 100px 10px hsl(195deg, 85%, 35%);
  }

  .glass {
    background: radial-gradient(
      circle at bottom left,
      hsl(195, 78%, 18%, 0.2),
      hsl(200, 100%, 50%, 0.2) 100%
    );
    backdrop-filter: blur(6px);
    box-shadow: 0 0 100px 20px hsl(250deg, 50%, 70%);
    border-width: 1px;
    border-color: white;
    border-radius: 1em;
    z-index: 1;
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

  .hero-text {
    position: sticky;
    top: 0;
  }
</style>
