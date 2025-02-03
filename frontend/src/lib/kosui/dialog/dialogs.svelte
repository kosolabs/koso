<script module lang="ts">
  import type { Snippet } from "svelte";

  type Dialog = {
    resolve: (ok: boolean) => void;
    msg: Snippet | string;
  };
  const dialogs: Dialog[] = $state([]);

  function confirm(msg: Snippet | string) {
    return new Promise((resolve) => {
      dialogs.push({
        msg,
        resolve,
      });
    });
  }

  export const dialog = {
    confirm,
  };
</script>

<script lang="ts">
  function ok() {
    let dialog = dialogs.pop();
    dialog?.resolve(true);
  }

  function cancel() {
    let dialog = dialogs.pop();
    dialog?.resolve(false);
  }
</script>

{#each dialogs as dialog}
  <dialog open={true}>
    <p>
      {#if typeof dialog.msg === "string"}
        {dialog.msg}
      {:else}
        {@render dialog.msg()}
      {/if}
    </p>
    <button onclick={cancel}>Cancel</button>
    <button onclick={ok}>Ok</button>
  </dialog>
{/each}
