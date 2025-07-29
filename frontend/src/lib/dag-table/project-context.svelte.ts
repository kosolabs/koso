import { page } from "$app/state";
import type { AuthContext } from "$lib/auth.svelte";
import type { User } from "$lib/users";
import { getContext, setContext } from "svelte";
import * as Y from "yjs";
import { Koso } from "./koso.svelte";
import { KosoSocket } from "./socket.svelte";

export class ProjectContext {
  id: string;
  koso: Koso;
  socket: KosoSocket;
  name: string = "";
  users: User[] = $state([]);

  constructor(id: string, koso: Koso, socket: KosoSocket) {
    this.id = id;
    this.koso = koso;
    this.socket = socket;
  }
}

export function newProjectContext(auth: AuthContext): ProjectContext {
  const id = page.params.projectId;
  if (!id) throw new Error("Missing projectId slug");
  const koso = new Koso(id, new Y.Doc());
  const socket = new KosoSocket(auth, koso, id);
  window.koso = koso;
  window.Y = Y;
  return setProjectContext(new ProjectContext(id, koso, socket));
}

export function setProjectContext(ctx: ProjectContext): ProjectContext {
  return setContext<ProjectContext>(ProjectContext, ctx);
}

export function getProjectContext(): ProjectContext {
  const ctx = getContext<ProjectContext>(ProjectContext);
  if (!ctx) throw new Error("ProjectContext is undefined");
  return ctx;
}
