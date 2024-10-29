import { logout_on_authentication_error } from "./errors";
import { version } from "$app/environment";
import { auth } from "./auth.svelte";

export type ErrorResponseBody = {
  status: number;
  details: ErrorDetail[];
};

export type ErrorDetail = {
  // Terse, stable, machine readable error reason.
  // e.g. NO_STOCK
  reason: string;
  msg: string;
};

export class KosoError extends Error {
  // Status code in number form. e.g. 400, 500
  status: number;
  details: ErrorDetail[];

  constructor({ status, details }: { status: number; details: ErrorDetail[] }) {
    const cause = details.map((d) => `(${d.reason}) ${d.msg}`).join(", ");
    super(`(${status}: [${cause})]`);
    this.status = status;
    this.details = details;
  }

  hasReason(reason: string): boolean {
    return this.details.find((d) => d.reason === reason) !== undefined;
  }
}

export function headers() {
  return {
    ...auth.headers(),
    "koso-client-version": version,
  };
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
    if (error.status !== response.status) {
      console.warn(
        `Error status (${error.status}) does not match response status ${response.status}`,
        error,
        response,
      );
    }
    throw new KosoError({
      status: error.status,
      details: error.details,
    });
  }
  throw new KosoError({
    status: response.status,
    details: [
      {
        reason: "UNKNOWN",
        msg: "No error details present.",
      },
    ],
  });
}
