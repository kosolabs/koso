import { version } from "$app/environment";
import { AuthContext } from "./auth.svelte";

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

export function headers(ctx: AuthContext) {
  return {
    ...ctx.headers(),
    "koso-client-version": version,
  };
}

/**
 * Returns the response body as json or throws if the response is not OK.
 *
 * @throws {KosoError}
 */
export async function parseResponse<T>(
  ctx: AuthContext,
  response: Response,
): Promise<T> {
  if (response.ok) {
    return response.json();
  }

  let err: KosoError;
  if (response.headers.get("Content-Type") === "application/json") {
    const error: ErrorResponseBody = await response.json();
    if (error.status !== response.status) {
      console.warn(
        `Error status (${error.status}) does not match response status ${response.status}`,
        error,
        response,
      );
    }
    err = new KosoError({
      status: error.status,
      details: error.details,
    });
  } else {
    err = new KosoError({
      status: response.status,
      details: [
        {
          reason: "UNKNOWN",
          msg: "No error details present.",
        },
      ],
    });
  }

  handleAuthErrors(ctx, err, response);

  throw err;
}

function handleAuthErrors(
  ctx: AuthContext,
  err: KosoError,
  response: Response,
) {
  const AUTHENTICATION_ERROR = 401;
  if (response.status === AUTHENTICATION_ERROR) {
    console.debug(
      "Response failed with an unathentication error (401). Logging user out.",
      response,
      err,
    );
    ctx.logout();
  }
}
