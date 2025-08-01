<script lang="ts">
  import { markdown } from "@codemirror/lang-markdown";
  import { EditorState } from "@codemirror/state";
  import { oneDarkTheme } from "@codemirror/theme-one-dark";
  import { type DOMEventHandlers } from "@codemirror/view";
  import { EditorView, basicSetup } from "codemirror";
  import type { ClassName } from "kosui";
  import { mode } from "mode-watcher";
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
  let editor: EditorView | undefined;

  export function focus() {
    editor?.focus();
  }

  function createEditor() {
    if (!yText.doc) throw new Error("Y.Text's doc was not initialized.");
    const dummyAwareness = new Awareness(yText.doc);
    return new EditorView({
      state: EditorState.create({
        doc: yText.toString(),
        extensions: [
          basicSetup,
          markdown(),
          yCollab(yText, dummyAwareness),
          EditorView.domEventHandlers(handlers),
          EditorView.lineWrapping,
          mode.current === "dark" ? oneDarkTheme : [],
        ],
      }),
      parent: el,
    });
  }

  $effect(() => {
    if (el) {
      editor = createEditor();
      return () => editor?.destroy();
    }
  });
</script>

<div
  class={twMerge("h-full rounded-md", className)}
  bind:this={el}
  {...restProps}
></div>
