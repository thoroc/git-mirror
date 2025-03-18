#!/usr/bin/env -S deno run --allow-env --allow-read --allow-write --allow-run
import { gitMirrorCommand } from "./cli/command.ts";

if (import.meta.main) {
  await gitMirrorCommand.parse(Deno.args);
}
