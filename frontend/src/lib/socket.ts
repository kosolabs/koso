import { Koso } from "$lib/koso";

export class KosoSocket {
  socket: WebSocket | null = null;
  shutdown: boolean = false;
  socketPingInterval: ReturnType<typeof setTimeout> | null = null;
  reconnectBackoffMs: number | null = null;
  offlineTimeout: ReturnType<typeof setTimeout> | null = null;
  offlineHandler: (event: Event) => void;
  onlineHandler: (event: Event) => Promise<void>;
  koso: Koso;
  token: () => string | null;
  projectId: string;
  onUnauthorized: () => void;
  onOnline: () => void;
  onOffline: () => void;

  constructor(
    koso: Koso,
    projectId: string,
    token: () => string | null,
    onUnauthorized: () => void,
    onOnline: () => void,
    onOffline: () => void,
  ) {
    this.koso = koso;
    this.projectId = projectId;
    this.token = token;
    this.onUnauthorized = onUnauthorized;
    this.onOnline = onOnline;
    this.onOffline = onOffline;

    this.#setOffline();

    this.onlineHandler = this.#handleOnline.bind(this);
    window.addEventListener("online", this.onlineHandler);
    this.offlineHandler = this.#handleOffline.bind(this);
    window.addEventListener("offline", this.offlineHandler);
    this.socketPingInterval = setInterval(
      () => {
        if (this.socket && this.socket.readyState == WebSocket.OPEN) {
          this.socket.send("");
        }
      },
      (45 + 20 * Math.random()) * 1000,
    );
  }

  async openWebSocket() {
    const token = this.token();
    if (!token) throw new Error("User is unauthorized");
    if (
      this.socket &&
      (this.socket.readyState == WebSocket.OPEN ||
        this.socket.readyState == WebSocket.CONNECTING)
    ) {
      console.log("Socket already connected");
      return;
    }
    if (this.shutdown) {
      return;
    }

    const host = location.origin.replace(/^http/, "ws");
    const wsUrl = `${host}/api/ws/projects/${this.projectId}`;
    const socket = new WebSocket(wsUrl, ["bearer", token]);
    this.socket = socket;
    socket.binaryType = "arraybuffer";

    socket.onopen = (event) => {
      console.log("WebSocket opened", event);
      this.#setOnline();
      this.koso.handleClientMessage((update) => {
        if (socket.readyState == WebSocket.OPEN) {
          socket.send(update);
        } else {
          console.warn(
            "Tried to send to non-open socket, discarded message",
            socket,
          );
        }
      });
    };

    socket.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        this.koso.handleServerMessage(new Uint8Array(event.data));
      } else {
        console.log("Received text frame from server:", event.data);
      }
    };
    socket.onerror = (event) => {
      console.log("WebSocket errored", event);
      // Errors also trigger onclose events so handle everything there.
    };
    socket.onclose = (event) => {
      // Sometimes onclose events are delayed while, in the meantime,
      // a new socket was opened.
      if (this.socket && this.socket != socket) {
        console.log("Socket already reopened");
        return;
      }

      this.#setOffline();
      if (this.shutdown) {
        console.log(
          `WebSocket closed in onDestroy. Code: ${event.code}, Reason: '${event.reason}' Will not try to reconnect`,
          event,
        );
        return;
      }

      const UNAUTHORIZED = 3000;
      if (event.code === UNAUTHORIZED) {
        console.log(
          `Unauthorized, WebSocket closed. Code: ${event.code}, Reason: '${event.reason}'. `,
          event,
        );
        this.#setShutdown();
        this.onUnauthorized();
        return;
      }

      const OVERLOADED = 1013;
      let backoffMs;
      if (event.code === OVERLOADED) {
        // In case of overload, don't retry aggressively.
        backoffMs = this.#backoffOnReconnect(30000);
        console.log(
          `Overloaded WebSocket closed. Code: ${event.code}, Reason: '${event.reason}'. Will try to reconnect in ${backoffMs} ms.`,
          event,
        );
      } else {
        backoffMs = this.#backoffOnReconnect();
        console.log(
          `WebSocket closed. Code: ${event.code}, Reason: '${event.reason}'. Will try to reconnect in ${backoffMs} ms.`,
          event,
        );
      }

      // Try to reconnect.
      setTimeout(async () => {
        await this.openWebSocket();
      }, backoffMs);
    };

    while (socket.readyState == WebSocket.CONNECTING) {
      await new Promise((r) => setTimeout(r, 100));
    }
  }

  async #handleOnline(event: Event) {
    console.log("Online.", event);
    await this.openWebSocket();
  }

  #handleOffline(event: Event) {
    console.log("Offline.", event);
    this.#setOffline(1);
    if (this.socket) {
      const socket = this.socket;
      this.socket = null;
      socket.close(1000, "Went offline");
    }
  }

  closeAndShutdown(code: number, reason: string) {
    const socket = this.socket;
    this.#setShutdown();
    if (socket) {
      socket.close(code, reason);
    }
  }

  #setShutdown() {
    this.shutdown = true;
    if (this.socketPingInterval) {
      clearInterval(this.socketPingInterval);
    }
    if (this.offlineTimeout) {
      clearTimeout(this.offlineTimeout);
    }
    window.removeEventListener("online", this.onlineHandler);
    window.removeEventListener("offline", this.offlineHandler);
    this.socket = null;
  }

  #setOffline(alertDelayMs = 14000) {
    if (!this.offlineTimeout && !this.shutdown) {
      // Delay showing the offline alert for a little bit
      // to avoid flashing an alert due to transient events.
      // e.g. server restarts.
      this.offlineTimeout = setTimeout(() => {
        if (this.offlineTimeout && !this.shutdown) {
          this.onOffline();
        }
      }, alertDelayMs);
    }
  }

  #setOnline() {
    this.reconnectBackoffMs = null;
    if (this.offlineTimeout) {
      clearTimeout(this.offlineTimeout);
      this.offlineTimeout = null;
    }
    this.onOnline();
  }

  #backoffOnReconnect(min: number = 0): number {
    let base = this.reconnectBackoffMs ? this.reconnectBackoffMs * 1.5 : 400;
    // Don't let backoff get too big (or too small).
    base = Math.max(Math.min(60000, base), min);
    // Add some jitter
    this.reconnectBackoffMs = base + base * 0.3 * Math.random();
    return this.reconnectBackoffMs;
  }
}
