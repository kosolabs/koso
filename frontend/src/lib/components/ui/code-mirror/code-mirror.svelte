<script lang="ts">
  import type { ClassName } from "$lib/kosui/utils";
  import { markdown } from "@codemirror/lang-markdown";
  import { EditorState } from "@codemirror/state";
  import { type DOMEventHandlers } from "@codemirror/view";
  import { EditorView, basicSetup } from "codemirror";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import { yCollab } from "y-codemirror.next";
  import { Awareness } from "y-protocols/awareness.js";
  import * as Y from "yjs";

  type Props = {
    yText: Y.Text;
    handlers?: DOMEventHandlers<unknown>;
  } & ClassName &
    HTMLAttributes<HTMLDivElement>;
  let {
    yText,
    handlers = {},
    class: className,
    ...restProps
  }: Props = $props();

  let el: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (el && yText.doc) {
      const dummyAwareness = new Awareness(yText.doc);
      const editor = new EditorView({
        state: EditorState.create({
          doc: yText.toString(),
          extensions: [
            basicSetup,
            markdown(),
            yCollab(yText, dummyAwareness),
            EditorView.domEventHandlers(handlers),
          ],
        }),
        parent: el,
      });

      return () => {
        editor.destroy();
      };
    }
  });
</script>

<div
  class={twMerge("h-full rounded-md", className)}
  bind:this={el}
  {...restProps}
></div>
