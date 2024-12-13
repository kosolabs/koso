import { Koso } from "$lib/koso.svelte";
import { auth } from "./auth.svelte";
import { version } from "$app/environment";

export class KosoSocket {
  socket: WebSocket | null = null;
  shutdown: boolean = false;
  socketPingInterval: ReturnType<typeof setTimeout> | null = null;
  reconnectBackoffMs: number | null = null;
  lastReconnectTime: number | null = null;
  offlineTimeout: ReturnType<typeof setTimeout> | null = null;
  offlineHandler: (event: Event) => void;
  onlineHandler: (event: Event) => Promise<void>;
  koso: Koso;
  projectId: string;
  onUnauthorized: () => void;
  onOnline: () => void;
  onOffline: () => void;

  constructor(
    koso: Koso,
    projectId: string,
    onUnauthorized: () => void,
    onOnline: () => void,
    onOffline: () => void,
  ) {
    this.koso = koso;
    this.projectId = projectId;
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
    if (
      this.socket &&
      (this.socket.readyState == WebSocket.OPEN ||
        this.socket.readyState == WebSocket.CONNECTING)
    ) {
      console.debug("Socket already connected");
      return;
    }
    if (this.shutdown) {
      return;
    }

    const host = location.origin.replace(/^http/, "ws");
    const wsUrl = `${host}/api/ws/projects/${this.projectId}`;
    const socket = new WebSocket(wsUrl, [
      "bearer",
      auth.token,
      "koso-client-version",
      version,
    ]);
    this.socket = socket;
    socket.binaryType = "arraybuffer";

    socket.onopen = (event) => {
      console.debug("WebSocket opened", event);
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
        console.debug("Received text frame from server:", event.data);
      }
    };
    socket.onerror = (event) => {
      console.debug("WebSocket errored", event);
      // Errors also trigger onclose events so handle everything there.
    };
    socket.onclose = (event) => {
      // Sometimes onclose events are delayed while, in the meantime,
      // a new socket was opened.
      if (this.socket && this.socket != socket) {
        console.debug("Socket already reopened");
        return;
      }

      this.#setOffline();
      if (this.shutdown) {
        console.debug(
          `WebSocket closed in onDestroy. Code: ${event.code}, Reason: '${event.reason}' Will not try to reconnect`,
          event,
        );
        return;
      }

      const UNAUTHORIZED = 3000;
      if (event.code === UNAUTHORIZED) {
        console.debug(
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
        console.debug(
          `Overloaded WebSocket closed. Code: ${event.code}, Reason: '${event.reason}'. Will try to reconnect in ${backoffMs} ms.`,
          event,
        );
      } else {
        backoffMs = this.#backoffOnReconnect();
        console.debug(
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
    console.debug("Online.", event);
    await this.openWebSocket();
  }

  #handleOffline(event: Event) {
    console.debug("Offline.", event);
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
    if (this.offlineTimeout) {
      clearTimeout(this.offlineTimeout);
      this.offlineTimeout = null;
    }
    this.onOnline();
  }

  #backoffOnReconnect(min: number = 0): number {
    const maxBackoffMs = 60 * 1000;
    // Reset the backoff time if the last backoff attempt was long ago,
    // removing any memory of long past strings of failures and allowing
    // clients to rapidly reconnect when interrupted.
    if (Date.now() - (this.lastReconnectTime || 0) > maxBackoffMs * 4) {
      this.reconnectBackoffMs = null;
    }

    let base = this.reconnectBackoffMs ? this.reconnectBackoffMs * 1.5 : 400;
    // Don't let backoff get too big (or too small).
    base = Math.max(Math.min(maxBackoffMs, base), min);
    // Add some jitter
    this.reconnectBackoffMs = base + base * 0.3 * Math.random();
    this.lastReconnectTime = Date.now();
    return this.reconnectBackoffMs;
  }
}
