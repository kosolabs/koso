<script>
  import { CodeMirror } from "$lib/components/ui/code-mirror";
  import * as Y from "yjs";

  const yDoc = new Y.Doc();
  const yText = yDoc.getText("myYText");
</script>

<CodeMirror {yText} />
