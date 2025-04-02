import type { MarkedToken } from "marked";
import { getContext, setContext, type Snippet } from "svelte";

type Token = {
  [T in TokenType]: Extract<MarkedToken, { type: T }>;
};

export type MarkdownComponentProps<T> = {
  token: T;
  children: Snippet;
};

export type TokenType = MarkedToken["type"];

export type TokenRenderer<T extends TokenType> = Snippet<
  [MarkdownComponentProps<Token[T]>]
>;

type Renderers = {
  [T in TokenType]?: TokenRenderer<T>;
};

export class MarkdownContext {
  #renderers: Renderers;

  constructor(renderers: Renderers) {
    this.#renderers = renderers;
  }

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

export function newMarkdownContext(renderers: Renderers) {
  return setMarkdownContext(new MarkdownContext(renderers));
}

export function setMarkdownContext(state: MarkdownContext): MarkdownContext {
  return setContext<MarkdownContext>(MarkdownContext, state);
}

export function getMarkdownContext(): MarkdownContext {
  return getContext<MarkdownContext>(MarkdownContext);
}
