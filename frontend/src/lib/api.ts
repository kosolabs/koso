import { logout_on_authentication_error } from "./errors";

export type ErrorResponseBody = {
  status: number;
  reason: string;
  msg: string;
};

export class KosoError extends Error {
  // Status code in number form. e.g. 400, 500
  status: number;
  // Terse, stable, machine readable error reason.
  // e.g. NO_STOCK
  reason: string;

  constructor({
    status,
    reason,
    msg,
  }: {
    status: number;
    reason: string;
    msg: string;
  }) {
    super(`${reason} (${status}: ${msg})`);
    this.status = status;
    this.reason = reason;
  }
}

/**
 * Returns the response body as json or throws if the response is not OK.
 * @throws {KosoError}
 */
export async function parse_response<T>(response: Response): Promise<T> {
  if (response.ok) {
    const ret: T = await response.json();
    return ret;
  }

  logout_on_authentication_error(response);
  if (response.headers.get("Content-Type") === "application/json") {
    const error: ErrorResponseBody = await response.json();
    throw new KosoError({
      status: error.status,
      reason: error.reason,
      msg: error.msg,
    });
  }
  throw new KosoError({
    status: response.status,
    reason: "UNKNOWN",
    msg: "No error details present.",
  });
}
