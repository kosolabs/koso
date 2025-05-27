import type { MarkedToken, Tokens } from "marked";
import { getContext, setContext, type Snippet } from "svelte";

type Token = {
  [T in TokenType]: Extract<MarkedToken, { type: T }>;
};

export type MarkdownComponentProps<T> = {
  token: T;
  children: Snippet;
};

export type TokenType = MarkedToken["type"];

type TokenRenderer<T> = Snippet<[MarkdownComponentProps<T>]>;

type Renderers = {
  [T in TokenType]?: TokenRenderer<Token[T]>;
};

export class MarkdownContext {
  #renderers: Renderers;
  #tableCellRenderer: TokenRenderer<Tokens.TableCell>;

  constructor(
    renderers: Renderers,
    tableCellRenderer: TokenRenderer<Tokens.TableCell>,
  ) {
    this.#renderers = renderers;
    this.#tableCellRenderer = tableCellRenderer;
  }

  get renderers() {
    return this.#renderers;
  }

  get tableCellRenderer() {
    return this.#tableCellRenderer;
  }

  getRenderer<T extends TokenType>(
    type: T,
  ): TokenRenderer<Token[T]> | undefined {
    if (!this.#renderers[type]) {
      console.warn("No renderer registered for", type);
    }
    return this.#renderers[type];
  }
}

export function setMarkdownContext(state: MarkdownContext): MarkdownContext {
  return setContext<MarkdownContext>(MarkdownContext, state);
}

export function getMarkdownContext(): MarkdownContext {
  const ctx = getContext<MarkdownContext>(MarkdownContext);
  if (!ctx) throw new Error("MarkdownContext is undefined");
  return ctx;
}
