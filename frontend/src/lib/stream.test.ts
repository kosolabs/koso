import { describe, expect, it, vi } from "vitest";
import { bytesFromReader, linesFromBytes } from "./stream";

describe("stream tests", () => {
  function createMockReader(
    stream: string[],
  ): ReadableStreamDefaultReader<Uint8Array> {
    let chunkIndex = 0;

    return {
      read: async () => {
        if (chunkIndex >= stream.length) {
          return { done: true, value: undefined };
        }
        const chunk = new TextEncoder().encode(stream[chunkIndex++]);
        return { done: false, value: chunk };
      },
      releaseLock: () => {},
    } as ReadableStreamDefaultReader<Uint8Array>;
  }

  describe("bytesFromReader", () => {
    it("yields all chunks from reader", async () => {
      const stream = ["hello", "world", "foo"];
      const reader = createMockReader(stream);

      const actual = [];
      for await (const chunk of bytesFromReader(reader)) {
        actual.push(new TextDecoder().decode(chunk));
      }

      expect(actual).toEqual(["hello", "world", "foo"]);
    });

    it("handles empty reader", async () => {
      const stream: string[] = [];
      const reader = createMockReader(stream);

      const actual = [];
      for await (const chunk of bytesFromReader(reader)) {
        actual.push(chunk);
      }

      expect(actual).toEqual([]);
    });

    it("handles single chunk", async () => {
      const stream = ["single chunk"];
      const reader = createMockReader(stream);

      const actual = [];
      for await (const chunk of bytesFromReader(reader)) {
        actual.push(new TextDecoder().decode(chunk));
      }

      expect(actual).toEqual(["single chunk"]);
    });

    it("releases lock when done", async () => {
      const stream = ["test"];
      const reader = createMockReader(stream);
      reader.releaseLock = vi.fn();

      const chunks = [];
      for await (const chunk of bytesFromReader(reader)) {
        chunks.push(chunk);
      }

      expect(reader.releaseLock).toHaveBeenCalledOnce();
    });

    it("releases lock on error", async () => {
      const reader = createMockReader([]);
      reader.read = async () => {
        throw new Error("Read error");
      };
      reader.releaseLock = vi.fn();

      try {
        for await (const _chunk of bytesFromReader(reader)) {
          // Should not reach here
        }
      } catch (error) {
        expect((error as Error).message).toBe("Read error");
      }

      expect(reader.releaseLock).toHaveBeenCalledOnce();
    });
  });

  describe("linesFromBytes", () => {
    async function* createMockByteStream(
      chunks: string[],
    ): AsyncGenerator<Uint8Array, void, unknown> {
      for (const chunk of chunks) {
        yield new TextEncoder().encode(chunk);
      }
    }

    it("yields complete lines from byte stream", async () => {
      const chunks = ["hello\nworld\n", "foo\nbar"];
      const stream = createMockByteStream(chunks);

      const actual = [];
      for await (const line of linesFromBytes(stream)) {
        actual.push(line);
      }

      expect(actual).toEqual(["hello", "world", "foo", "bar"]);
    });

    it("handles incomplete lines across chunks", async () => {
      const chunks = ["hel", "lo\nwor", "ld\nfoo"];
      const stream = createMockByteStream(chunks);

      const actual = [];
      for await (const line of linesFromBytes(stream)) {
        actual.push(line);
      }

      expect(actual).toEqual(["hello", "world", "foo"]);
    });

    it("yields final line without newline", async () => {
      const chunks = ["line1\nline2\nline3"];
      const stream = createMockByteStream(chunks);

      const actual = [];
      for await (const line of linesFromBytes(stream)) {
        actual.push(line);
      }

      expect(actual).toEqual(["line1", "line2", "line3"]);
    });

    it("handles empty stream", async () => {
      const chunks: string[] = [];
      const stream = createMockByteStream(chunks);

      const actual = [];
      for await (const line of linesFromBytes(stream)) {
        actual.push(line);
      }

      expect(actual).toEqual([]);
    });

    it("handles stream with only newlines", async () => {
      const chunks = ["\n\n\n"];
      const stream = createMockByteStream(chunks);

      const actual = [];
      for await (const line of linesFromBytes(stream)) {
        actual.push(line);
      }

      expect(actual).toEqual(["", "", ""]);
    });

    it("handles single chunk with multiple lines", async () => {
      const chunks = ["line1\nline2\nline3\n"];
      const stream = createMockByteStream(chunks);

      const actual = [];
      for await (const line of linesFromBytes(stream)) {
        actual.push(line);
      }

      expect(actual).toEqual(["line1", "line2", "line3"]);
    });

    it("handles chunk ending with newline", async () => {
      const chunks = ["hello\n", "world\n"];
      const stream = createMockByteStream(chunks);

      const actual = [];
      for await (const line of linesFromBytes(stream)) {
        actual.push(line);
      }

      expect(actual).toEqual(["hello", "world"]);
    });

    it("handles UTF-8 characters across chunk boundaries", async () => {
      const chunks = ["héll", "ô\nwørld"];
      const stream = createMockByteStream(chunks);

      const actual = [];
      for await (const line of linesFromBytes(stream)) {
        actual.push(line);
      }

      expect(actual).toEqual(["héllô", "wørld"]);
    });
  });

  describe("chain bytesFromReader to linesFromBytes", () => {
    it("yields complete lines from a stream", async () => {
      const stream = ["hello\nworld\n", "foo\nbar"];

      const actual = await Array.fromAsync(
        linesFromBytes(bytesFromReader(createMockReader(stream))),
      );

      expect(actual).toEqual(["hello", "world", "foo", "bar"]);
    });

    it("handles stream with no newlines", async () => {
      const stream = ["hello ", "world"];

      const actual = await Array.fromAsync(
        linesFromBytes(bytesFromReader(createMockReader(stream))),
      );

      expect(actual).toEqual(["hello world"]);
    });

    it("handles empty stream", async () => {
      const stream: string[] = [];

      const actual = await Array.fromAsync(
        linesFromBytes(bytesFromReader(createMockReader(stream))),
      );

      expect(actual).toEqual([]);
    });

    it("handles stream with only newlines", async () => {
      const stream = ["\n\n\n"];

      const actual = await Array.fromAsync(
        linesFromBytes(bytesFromReader(createMockReader(stream))),
      );

      expect(actual).toEqual(["", "", ""]);
    });

    it("handles lines split across multiple chunks", async () => {
      const stream = ["hel", "lo\nwor", "ld\nfoo"];

      const actual = await Array.fromAsync(
        linesFromBytes(bytesFromReader(createMockReader(stream))),
      );

      expect(actual).toEqual(["hello", "world", "foo"]);
    });
  });
});
