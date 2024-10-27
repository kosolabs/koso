import { logout_on_authentication_error } from "./errors";

export type ErrorResponseBody = {
  status: number;
  reason: string;
  msg: string;
};

export class KosoError extends Error {
  status: number;
  reason: string;
  msg: string;

  constructor({
    status,
    reason,
    msg,
  }: {
    status: number;
    reason: string;
    msg: string;
  }) {
    super();
    this.status = status;
    this.reason = reason;
    this.msg = msg;
  }
}

export async function parse_response<T>(response: Response) {
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
