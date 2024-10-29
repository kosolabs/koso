import { auth } from "./auth.svelte";

export function logout_on_authentication_error(response: Response) {
  const AUTHENTICATION_ERROR = 401;
  if (response.status === AUTHENTICATION_ERROR) {
    console.debug(
      "Response failed with an unathentication error (401). Logging user out.",
      response,
    );
    auth.logout();
  }
}
