<script>
  import { Badge } from "$lib/kosui/badge";
  import { Button } from "$lib/kosui/button";
  import { Menu } from "lucide-svelte";
</script>

<div class="flex gap-5">
  <Badge content="👑" variant="plain">
    <Button>Button with Badge</Button>
  </Badge>

  <Badge content="2" verticalAlignment="bottom" horizontalAlignment="left">
    <Button>Button with Badge</Button>
  </Badge>

  <Badge content="3" verticalAlignment="bottom">
    <Button>Button with Badge</Button>
  </Badge>

  <Badge content="4" class="m-1.5">
    <Button icon={Menu} shape="circle" />
  </Badge>
</div>
