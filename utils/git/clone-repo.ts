import chalk from "npm:chalk";
import { findExecutable } from "../exec/find-exec.ts";

export const cloneRepo = async (
  repo: string,
  localRepo: string,
): Promise<void> => {
  const git = await findExecutable("git");

  const clone = new Deno.Command(git, {
    args: ["clone", repo, localRepo],
    stdin: "piped",
  });
  const child = await clone.spawn();
  const status = await child.status;

  if (!status.success) {
    console.error(chalk.bgRedBright(`Error cloning repository`));
  } else {
    console.log(
      chalk.bgGreenBright(`Repository cloned successfully to ${localRepo}`),
    );
  }
};
