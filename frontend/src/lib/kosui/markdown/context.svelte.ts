import type { MarkedToken } from "marked";
import { getContext, setContext, type Component, type Snippet } from "svelte";
import {
  MarkdownBlockquote,
  MarkdownBr,
  MarkdownCode,
  MarkdownCodespan,
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
  MarkdownText,
} from ".";

export type MarkdownComponentProps<T> = {
  token: T;
  children: Snippet;
};

export type TokenType = MarkedToken["type"];

type Token = {
  [T in TokenType]: Extract<MarkedToken, { type: T }>;
};

type TokenRenderer<T extends TokenType> = Component<{
  token: Token[T];
  children: Snippet;
}>;

type Renderers = {
  [T in TokenType]: TokenRenderer<T>;
};

export class MarkdownContext {
  #renderers: Renderers = {
    blockquote: MarkdownBlockquote,
    br: MarkdownBr,
    code: MarkdownCode,
    codespan: MarkdownCodespan,
    def: MarkdownDef,
    del: MarkdownDel,
    em: MarkdownEm,
    escape: MarkdownEscape,
    heading: MarkdownHeading,
    hr: MarkdownHr,
    html: MarkdownHtml,
    image: MarkdownImage,
    link: MarkdownLink,
    list_item: MarkdownListItem,
    list: MarkdownList,
    paragraph: MarkdownParagraph,
    space: MarkdownSpace,
    strong: MarkdownStrong,
    table: MarkdownTable,
    text: MarkdownText,
  };

  get renderers() {
    return this.#renderers;
  }

  getRenderer<T extends TokenType>(type: T): TokenRenderer<T> | undefined {
    if (!this.#renderers[type]) {
      console.warn("No renderer registered for", type);
    }
    return this.#renderers[type];
  }
}

export function newMarkdownContext() {
  return setMarkdownContext(new MarkdownContext());
}

export function setMarkdownContext(state: MarkdownContext): MarkdownContext {
  return setContext<MarkdownContext>(MarkdownContext, state);
}

export function getMarkdownContext(): MarkdownContext {
  return getContext<MarkdownContext>(MarkdownContext);
}
