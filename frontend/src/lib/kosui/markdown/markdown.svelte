<script module lang="ts">
  import {
    Lexer,
    type MarkedOptions,
    type MarkedToken,
    type Tokens,
  } from "marked";
  import type { Snippet } from "svelte";
  import {
    MarkdownBlockquote,
    MarkdownBr,
    MarkdownCode,
    MarkdownCodespan,
    MarkdownContext,
    MarkdownDef,
    MarkdownDel,
    MarkdownEm,
    MarkdownEscape,
    MarkdownHeading,
    MarkdownHr,
    MarkdownHtml,
    MarkdownImage,
    MarkdownLink,
    MarkdownList,
    MarkdownListItem,
    MarkdownParagraph,
    MarkdownSpace,
    MarkdownStrong,
    MarkdownTable,
    MarkdownTableCell,
    MarkdownText,
    MarkdownTokens,
    setMarkdownContext,
    type MarkdownComponentProps,
  } from ".";

  export type MarkdownProps = {
    value: string;
    options?: MarkedOptions;
    blockquote?: Snippet<[MarkdownComponentProps<Tokens.Blockquote>]>;
    br?: Snippet<[MarkdownComponentProps<Tokens.Br>]>;
    code?: Snippet<[MarkdownComponentProps<Tokens.Code>]>;
    codespan?: Snippet<[MarkdownComponentProps<Tokens.Codespan>]>;
    def?: Snippet<[MarkdownComponentProps<Tokens.Def>]>;
    del?: Snippet<[MarkdownComponentProps<Tokens.Del>]>;
    em?: Snippet<[MarkdownComponentProps<Tokens.Em>]>;
    escape?: Snippet<[MarkdownComponentProps<Tokens.Escape>]>;
    heading?: Snippet<[MarkdownComponentProps<Tokens.Heading>]>;
    hr?: Snippet<[MarkdownComponentProps<Tokens.Hr>]>;
    html?: Snippet<[MarkdownComponentProps<Tokens.HTML | Tokens.Tag>]>;
    image?: Snippet<[MarkdownComponentProps<Tokens.Image>]>;
    link?: Snippet<[MarkdownComponentProps<Tokens.Link>]>;
    listItem?: Snippet<[MarkdownComponentProps<Tokens.ListItem>]>;
    list?: Snippet<[MarkdownComponentProps<Tokens.List>]>;
    paragraph?: Snippet<[MarkdownComponentProps<Tokens.Paragraph>]>;
    space?: Snippet<[MarkdownComponentProps<Tokens.Space>]>;
    strong?: Snippet<[MarkdownComponentProps<Tokens.Strong>]>;
    tableCell?: Snippet<[MarkdownComponentProps<Tokens.TableCell>]>;
    table?: Snippet<[MarkdownComponentProps<Tokens.Table>]>;
    text?: Snippet<[MarkdownComponentProps<Tokens.Text>]>;
  };
</script>

<script lang="ts">
  let {
    value,
    options = {},
    blockquote = defaultBlockquote,
    br = defaultBr,
    code = defaultCode,
    codespan = defaultCodespan,
    def = defaultDef,
    del = defaultDel,
    em = defaultEm,
    escape = defaultEscape,
    heading = defaultHeading,
    hr = defaultHr,
    html = defaultHtml,
    image = defaultImage,
    link = defaultLink,
    listItem = defaultListItem,
    list = defaultList,
    paragraph = defaultParagraph,
    space = defaultSpace,
    strong = defaultStrong,
    tableCell = defaultTableCell,
    table = defaultTable,
    text = defaultText,
  }: MarkdownProps = $props();

  setMarkdownContext(
    new MarkdownContext(
      {
        blockquote,
        br,
        code,
        codespan,
        def,
        del,
        em,
        escape,
        heading,
        hr,
        html,
        image,
        link,
        list_item: listItem,
        list,
        paragraph,
        space,
        strong,
        table,
        text,
      },
      tableCell,
    ),
  );

  let tokens = $derived(Lexer.lex(value, options) as MarkedToken[]);
</script>

{#snippet defaultBlockquote(props: MarkdownComponentProps<Tokens.Blockquote>)}
  <MarkdownBlockquote {...props} />
{/snippet}
{#snippet defaultBr(props: MarkdownComponentProps<Tokens.Br>)}
  <MarkdownBr {...props} />
{/snippet}
{#snippet defaultCode(props: MarkdownComponentProps<Tokens.Code>)}
  <MarkdownCode {...props} />
{/snippet}
{#snippet defaultCodespan(props: MarkdownComponentProps<Tokens.Codespan>)}
  <MarkdownCodespan {...props} />
{/snippet}
{#snippet defaultDef(props: MarkdownComponentProps<Tokens.Def>)}
  <MarkdownDef {...props} />
{/snippet}
{#snippet defaultDel(props: MarkdownComponentProps<Tokens.Del>)}
  <MarkdownDel {...props} />
{/snippet}
{#snippet defaultEm(props: MarkdownComponentProps<Tokens.Em>)}
  <MarkdownEm {...props} />
{/snippet}
{#snippet defaultEscape(props: MarkdownComponentProps<Tokens.Escape>)}
  <MarkdownEscape {...props} />
{/snippet}
{#snippet defaultHeading(props: MarkdownComponentProps<Tokens.Heading>)}
  <MarkdownHeading {...props} />
{/snippet}
{#snippet defaultHr(props: MarkdownComponentProps<Tokens.Hr>)}
  <MarkdownHr {...props} />
{/snippet}
{#snippet defaultHtml(props: MarkdownComponentProps<Tokens.HTML | Tokens.Tag>)}
  <MarkdownHtml {...props} />
{/snippet}
{#snippet defaultImage(props: MarkdownComponentProps<Tokens.Image>)}
  <MarkdownImage {...props} />
{/snippet}
{#snippet defaultLink(props: MarkdownComponentProps<Tokens.Link>)}
  <MarkdownLink {...props} />
{/snippet}
{#snippet defaultListItem(props: MarkdownComponentProps<Tokens.ListItem>)}
  <MarkdownListItem {...props} />
{/snippet}
{#snippet defaultList(props: MarkdownComponentProps<Tokens.List>)}
  <MarkdownList {...props} />
{/snippet}
{#snippet defaultParagraph(props: MarkdownComponentProps<Tokens.Paragraph>)}
  <MarkdownParagraph {...props} />
{/snippet}
{#snippet defaultSpace(props: MarkdownComponentProps<Tokens.Space>)}
  <MarkdownSpace {...props} />
{/snippet}
{#snippet defaultStrong(props: MarkdownComponentProps<Tokens.Strong>)}
  <MarkdownStrong {...props} />
{/snippet}
{#snippet defaultTable(props: MarkdownComponentProps<Tokens.Table>)}
  <MarkdownTable {...props} />
{/snippet}
{#snippet defaultTableCell(props: MarkdownComponentProps<Tokens.TableCell>)}
  <MarkdownTableCell {...props} />
{/snippet}
{#snippet defaultText(props: MarkdownComponentProps<Tokens.Text>)}
  <MarkdownText {...props} />
{/snippet}

<MarkdownTokens {tokens} />
