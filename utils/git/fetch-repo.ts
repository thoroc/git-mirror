import { colors } from "jsr:@cliffy/ansi@^1.0.0-rc.7/colors";
import { findExecutable } from "../exec/find-exec.ts";

export const fetchRepo = async (localRepo: string): Promise<void> => {
  const git = await findExecutable("git");

  const pull = new Deno.Command(git, {
    cwd: localRepo,
    args: ["fetch", "origin"],
    stdin: "piped",
  });
  const child = await pull.spawn();
  const status = await child.status;

  if (!status.success) {
    console.error(colors.bgBrightRed(`Error fetching repository`));
  } else {
    console.log(colors.bgBrightGreen(`Repository fetching successfully`));
  }
};
