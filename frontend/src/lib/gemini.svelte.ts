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
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        const chunk = decoder.decode(value, { stream: true });
        const lines = chunk.split("\n");
        for (const line of lines) {
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
      }
    } finally {
      reader.releaseLock();
    }
    this.running = false;
    return response;
  }
}
