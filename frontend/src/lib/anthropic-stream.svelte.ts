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
  running = $state(true);
  stream = $state("");
  response: Promise<Response>;

  constructor(input: RequestInfo | URL, init?: RequestInit) {
    this.response = this.fetch(input, init);
  }

  private async fetch(
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
              this.stream = "";
            } else if (isAnthropicContentBlockDelta(data)) {
              this.stream += data.delta.text;
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
