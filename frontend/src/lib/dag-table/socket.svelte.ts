import { version } from "$app/environment";
import { auth } from "$lib/auth.svelte";
import { Koso } from ".";

export class KosoSocket {
  #unauthorized: boolean = $state(false);
  #offline: boolean = $state(false);
  #socket: WebSocket | null = null;
  #shutdown: boolean = false;
  #offlineTimeout: number | undefined;
  #reconnectBackoffMs: number | null = null;
  #lastReconnectTime: number | null = null;
  #koso: Koso;
  #projectId: string;

  constructor(koso: Koso, projectId: string) {
    this.#koso = koso;
    this.#projectId = projectId;

    this.#setOffline();

    $effect(() => {
      const handleOnline = this.#handleOnline.bind(this);
      const handleOffline = this.#handleOffline.bind(this);

      window.addEventListener("online", handleOnline);
      window.addEventListener("offline", handleOffline);

      const socketPingInterval = window.setInterval(
        () => {
          if (this.#socket && this.#socket.readyState === WebSocket.OPEN) {
            this.#socket.send("");
          }
        },
        (45 + 20 * Math.random()) * 1000,
      );

      this.#openWebSocket();

      return () => {
        window.removeEventListener("online", handleOnline);
        window.removeEventListener("offline", handleOffline);
        window.clearInterval(socketPingInterval);
        this.#closeAndShutdown(1000, "Closed in onDestroy.");
      };
    });
  }

  /**
   * True if the socket failed due to the user being unauthorized for the given
   * project.
   *
   * Note: False may indicate the user is authorized, but it may also indicate
   * that the socket has not connected yet. For example, the user is offline.
   */
  get unauthorized(): boolean {
    return this.#unauthorized;
  }

  /**
   * True if the socket is not connected
   *
   * Note: False may indicate the socket is connected, but it may also indicate
   * that the socket is trying to reconnect. To avoid fipping between states
   * rapidly, there's a delay in marking the socket of offline. See
   * `#setOffline`.
   */
  get offline(): boolean {
    return this.#offline;
  }

  #openWebSocket() {
    if (
      this.#socket &&
      (this.#socket.readyState === WebSocket.OPEN ||
        this.#socket.readyState === WebSocket.CONNECTING)
    ) {
      console.debug("Socket already connected");
      return;
    }
    if (this.#shutdown) {
      return;
    }

    const host = location.origin.replace(/^http/, "ws");
    const wsUrl = `${host}/api/ws/projects/${this.#projectId}`;
    const socket = new WebSocket(wsUrl, [
      "bearer",
      auth.token,
      "koso-client-version",
      version,
    ]);
    this.#socket = socket;
    socket.binaryType = "arraybuffer";

    socket.onopen = (event) => {
      console.debug("WebSocket opened", event);
      this.#setOnline();
      this.#koso.setSendAndSync((update) => {
        if (socket.readyState === WebSocket.OPEN) {
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
        this.#koso.receive(new Uint8Array(event.data));
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
      if (this.#socket && this.#socket !== socket) {
        console.debug("Socket already reopened");
        return;
      }

      this.#setOffline();
      if (this.#shutdown) {
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
        this.#closeAndShutdown(
          1000,
          "Responding to Unauthorized server closure.",
        );
        this.#unauthorized = true;
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
      window.setTimeout(() => {
        this.#openWebSocket();
      }, backoffMs);
    };
  }

  #closeAndShutdown(code?: number, reason?: string) {
    this.#shutdown = true;
    clearTimeout(this.#offlineTimeout);
    this.#offlineTimeout = undefined;
    if (this.#socket) {
      console.log("Closing socket.", reason);
      this.#socket.close(code, reason);
      this.#socket = null;
    }
  }

  #handleOnline(event: Event) {
    console.debug("Online.", event);
    this.#openWebSocket();
  }

  #handleOffline(event: Event) {
    console.debug("Offline.", event);
    this.#setOffline(1);
    if (this.#socket) {
      this.#socket.close(1000, "Went offline");
      this.#socket = null;
    }
  }

  #setOffline(alertDelayMs = 14000) {
    if (!this.#offlineTimeout && !this.#shutdown) {
      // Delay showing the offline alert for a little bit
      // to avoid flashing an alert due to transient events.
      // e.g. server restarts.
      this.#offlineTimeout = window.setTimeout(() => {
        if (this.#offlineTimeout && !this.#shutdown) {
          this.#offline = true;
        }
      }, alertDelayMs);
    }
  }

  #setOnline() {
    if (this.#offlineTimeout) {
      clearTimeout(this.#offlineTimeout);
      this.#offlineTimeout = undefined;
    }
    this.#offline = false;
  }

  #backoffOnReconnect(min: number = 0): number {
    const maxBackoffMs = 60 * 1000;
    // Reset the backoff time if the last backoff attempt was long ago,
    // removing any memory of long past strings of failures and allowing
    // clients to rapidly reconnect when interrupted.
    if (Date.now() - (this.#lastReconnectTime || 0) > maxBackoffMs * 4) {
      this.#reconnectBackoffMs = null;
    }

    let base = this.#reconnectBackoffMs ? this.#reconnectBackoffMs * 1.5 : 400;
    // Don't let backoff get too big (or too small).
    base = Math.max(Math.min(maxBackoffMs, base), min);
    // Add some jitter
    this.#reconnectBackoffMs = base + base * 0.3 * Math.random();
    this.#lastReconnectTime = Date.now();
    return this.#reconnectBackoffMs;
  }
}
