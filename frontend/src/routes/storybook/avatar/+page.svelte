<script lang="ts">
  import { Avatar } from "$lib/kosui/avatar";
  import { colors, variants } from "$lib/kosui/base";
  import { UserRound } from "@lucide/svelte";
</script>

<div class="flex gap-4 rounded-lg border p-4">
  {#each variants as variant (variant)}
    {#each colors as color (color)}
      <Avatar
        {variant}
        {color}
        src="https://static.wikia.nocookie.net/dilbert/images/6/60/Boss.PNG"
      />
    {/each}
  {/each}
</div>

<div class="flex gap-4 rounded-lg border p-4">
  {#each variants as variant (variant)}
    {#each colors as color (color)}
      <Avatar {variant} {color} shape="circle">FB</Avatar>
    {/each}
  {/each}
</div>

<div class="flex gap-4 rounded-lg border p-4">
  {#each variants as variant (variant)}
    {#each colors as color (color)}
      <Avatar {variant} {color} shape="square"><UserRound /></Avatar>
    {/each}
  {/each}
</div>
