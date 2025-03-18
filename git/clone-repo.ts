import { findExecutable } from "@scope/exec";
import { colors } from "jsr:@cliffy/ansi@^1.0.0-rc.7/colors";

export const cloneRepo = async (
  repo: string,
  localRepo: string,
): Promise<void> => {
  try {
    const git = await findExecutable("git");

    const clone = new Deno.Command(git, {
      args: ["clone", repo, localRepo],
      stdin: "piped",
    });
    const child = await clone.spawn();
    const status = await child.status;

    if (!status.success) {
      console.error(colors.bgBrightRed(`Error cloning repository`));
    } else {
      console.log(
        colors.bgBrightGreen(`Repository cloned successfully to ${localRepo}`),
      );
    }
  } catch (error) {
    console.error(colors.bgBrightRed(`Error cloning repository: ${error}`));
    Deno.exit(1);
  }
};
