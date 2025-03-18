export const HOME_DIR: string = Deno.env.get("HOME") ||
  Deno.env.get("USERPROFILE") || "";
