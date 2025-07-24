type AnthropicMessage = AnthropicContentBlockStart | AnthropicContentBlockDelta;

type AnthropicContentBlockStart = {
  type: "content_block_start";
  index: number;
  content_block: {
    type: "text";
    text: string;
  };
};

type AnthropicContentBlockDelta = {
  type: "content_block_delta";
  index: number;
  delta: {
    type: "text_delta";
    text: string;
  };
};

function isAnthropicContentBlockStart(
  data: AnthropicMessage,
): data is AnthropicContentBlockStart {
  return data.type === "content_block_start";
}

function isAnthropicContentBlockDelta(
  data: AnthropicMessage,
): data is AnthropicContentBlockDelta {
  return data.type === "content_block_delta";
}

export class AnthropicStream {
  running: boolean = $state(true);
  stream: string[] = $state([]);
  response: Promise<Response> | undefined;
  #lineSubscribers: ((line: string) => void)[] = [];
  #lineBuffer: LineBuffer = new LineBuffer((line) => {
    this.#lineSubscribers.forEach((subscriber) => subscriber(line));
  });

  onLine(subscriber: (line: string) => void): AnthropicStream {
    this.#lineSubscribers.push(subscriber);
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
            const data = JSON.parse(line.slice(5));
            if (isAnthropicContentBlockStart(data)) {
              this.stream = [];
            } else if (isAnthropicContentBlockDelta(data)) {
              this.#lineBuffer?.addToken(data.delta.text);
              this.stream.push(data.delta.text);
            }
          }
        }
      }
    } finally {
      this.#lineBuffer?.flush();
      reader.releaseLock();
    }
    this.running = false;
    return response;
  }
}

class LineBuffer {
  private buffer: string = "";
  private onLine: (line: string) => void;

  constructor(onLine: (line: string) => void) {
    this.onLine = onLine;
  }

  addToken(token: string): void {
    this.buffer += token;

    // Check for complete lines
    const lines = this.buffer.split("\n");

    // Keep the last incomplete line in the buffer
    this.buffer = lines.pop() || "";

    // Emit all complete lines
    for (const line of lines) {
      this.onLine(line);
    }
  }

  flush(): void {
    // Emit any remaining content as a final line
    if (this.buffer.length > 0) {
      this.onLine(this.buffer);
      this.buffer = "";
    }
  }
}
