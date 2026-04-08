import { login } from "../src/auth_service";

export function loginRoute(username: string) {
  return login(username);
}
