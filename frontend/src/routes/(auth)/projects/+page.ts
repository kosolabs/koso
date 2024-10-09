import { get } from "svelte/store";

/** @type {import('./$types').PageLoad} */
export async function load() {
  console.log("loaddinggg");
  const token = get((await import("$lib/auth")).token);
  return {
    projects: await (await import("$lib/projects")).fetchProjects(token),
  };
}
