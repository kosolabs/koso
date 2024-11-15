import { version } from "$app/environment";
import { toast } from "svelte-sonner";
import { auth } from "./auth.svelte";
import { logout_on_authentication_error } from "./errors";

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
 *
 * @throws {KosoError}
 */
export async function parse_response<T>(response: Response): Promise<T> {
  if (response.ok) {
    return response.json();
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
    const err = new KosoError({
      status: error.status,
      details: error.details,
    });

    if (err.hasReason("NOT_INVITED")) {
      console.debug(
        "Response failed, user is not invited. Logging user out.",
        response,
        err,
      );
      toast.warning("You don't have access to Koso.");
      auth.logout();
    }

    throw err;
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
