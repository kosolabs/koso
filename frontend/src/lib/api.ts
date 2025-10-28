import { version } from "$app/environment";
import { AuthContext } from "./auth.svelte";

export class KosoError extends Error {
  // Status code in number form. e.g. 400, 500
  status: number;
  title: string;
  detail: string;

  constructor(error: { status: number; title: string; detail: string }) {
    super(`${error.status} ${error.title}: ${error.detail}`);
    this.status = error.status;
    this.title = error.title;
    this.detail = error.detail;
  }

  hasReason(reason: string): boolean {
    return this.title === reason;
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
    const error: ConstructorParameters<typeof KosoError>[0] =
      await response.json();
    if (error.status !== response.status) {
      console.warn(
        `Error status (${error.status}) does not match response status ${response.status}`,
        error,
        response,
      );
    }
    err = new KosoError(error);
  } else {
    err = new KosoError({
      status: response.status,
      title: "UNKNOWN",
      detail: "No error details present.",
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
      "Response failed with an unauthentication error (401). Logging user out.",
      response,
      err,
    );
    ctx.logout();
  }
}
