import { bytesFromReader, linesFromBytes } from "./stream";

type GeminiMessage = {
  candidates: {
    content: {
      parts: {
        text: string;
      }[];
      role: string;
    };
  }[];
  usageMetadata: {
    promptTokenCount: number;
    candidatesTokenCount: number;
    totalTokenCount: number;
    promptTokensDetails: {
      modality: string;
      tokenCount: number;
    }[];
    thoughtsTokenCount: number;
  };
  modelVersion: string;
  responseId: string;
};

export class GeminiStream {
  running: boolean = $state(true);
  stream: string[] = $state([]);
  response: Promise<Response> | undefined;
  #subscribers: ((token: string) => void)[] = [];

  onLine(subscriber: (token: string) => void): GeminiStream {
    this.#subscribers.push(subscriber);
    return this;
  }

  fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
    return this.#fetch(input, init);
  }

  async #fetch(
    input: RequestInfo | URL,
    init?: RequestInit,
  ): Promise<Response> {
    const response = await fetch(input, init);
    if (!response.body) {
      throw new Error("Response body is null");
    }
    for await (const line of linesFromBytes(
      bytesFromReader(response.body.getReader()),
    )) {
      if (line.startsWith("data:")) {
        const data = JSON.parse(line.slice(5)) as GeminiMessage;
        for (const candidate of data.candidates) {
          for (const part of candidate.content.parts) {
            for (const subscriber of this.#subscribers) {
              subscriber(part.text);
            }
            this.stream.push(part.text);
          }
        }
      }
    }
    this.running = false;
    return response;
  }
}
