<script lang="ts">
  import type { MarkdownComponentProps } from "$lib/kosui/markdown";
  import type { ClassName } from "$lib/kosui/utils";
  import hljs from "highlight.js";
  import type { Tokens } from "marked";
  import mermaid from "mermaid";
  import { userPrefersMode as mode } from "mode-watcher";
  import { onMount } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { twMerge } from "tailwind-merge";
  import "./theme.css";
  mermaid.initialize({
    startOnLoad: false,
    theme: mode.current === "dark" ? "dark" : "default",
  });

  let {
    token,
    class: className,
    ...restProps
  }: MarkdownComponentProps<Tokens.Code> &
    ClassName &
    HTMLAttributes<HTMLPreElement> = $props();

  function render(token: Tokens.Code) {
    if (token.lang === "mermaid") {
      return { lang: "mermaid", code: token.text };
    } else if (token.lang && hljs.listLanguages().includes(token.lang)) {
      return {
        lang: `lang-${token.lang}`,
        code: hljs.highlight(token.text, { language: token.lang }).value,
      };
    } else {
      return {
        lang: "lang-plaintext",
        code: hljs.highlight(token.text, { language: "plaintext" }).value,
      };
    }
  }

  const { lang, code } = $derived(render(token));

  onMount(() => {
    mermaid.run();
  });
</script>

<!-- eslint-disable-next-line svelte/no-at-html-tags -->
<pre class={twMerge(lang, className)} {...restProps}>{@html code}</pre>
