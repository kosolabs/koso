<script module lang="ts">
  import { Lexer, type MarkedOptions, type MarkedToken } from "marked";
  import { MarkdownTokens, newMarkdownContext } from ".";

  export type MarkdownEditorProps = {
    value: string;
    options?: MarkedOptions;
  };
</script>

<script lang="ts">
  let { value = $bindable(""), options = {} }: MarkdownEditorProps = $props();

  newMarkdownContext();

  const lexer = new Lexer(options);
  let tokens = $derived(lexer.lex(value) as MarkedToken[]);
</script>

<MarkdownTokens {tokens} />
