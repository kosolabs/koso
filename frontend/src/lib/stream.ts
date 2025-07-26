export async function* bytesFromReader(
  reader: ReadableStreamDefaultReader<Uint8Array>,
): AsyncGenerator<Uint8Array, void, unknown> {
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      yield value;
    }
  } finally {
    reader.releaseLock();
  }
}

export async function* linesFromBytes(
  stream: AsyncGenerator<Uint8Array, void, unknown>,
): AsyncGenerator<string, void, unknown> {
  const decoder = new TextDecoder();
  let buffer = "";

  for await (const value of stream) {
    const chunk = decoder.decode(value, { stream: true });
    buffer += chunk;

    // Split on newlines and yield complete lines
    const lines = buffer.split("\n");

    // Keep the last incomplete line in the buffer
    buffer = lines.pop() || "";

    // Yield all complete lines
    for (const line of lines) {
      yield line;
    }
  }

  // Yield any remaining content as a final line
  if (buffer.length > 0) {
    yield buffer;
  }
}
