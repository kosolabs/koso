import { logout } from "./auth";

export function logout_on_authentication_error(response: Response) {
  if (response.status == 401) {
    console.log(
      "Response failed with an unathentication error (401). Logging user out.",
      response,
    );
    logout();
  }
}
