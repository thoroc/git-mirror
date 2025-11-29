import { HOME_DIR, VERSION } from "@scope/env";
import { Command } from "jsr:@cliffy/command@^1.0.0-rc.7";
import { cloneAction } from "./action.ts";

export const gitMirrorCommand = new Command()
  .name("git-mirror")
  .version(VERSION)
  .description("Clone/Fetch a Git repository into a 'Projects' directory")
  .arguments("<repo:string>")
  .option("-r, --root <rootDir>", "The root directory.", {
    default: `${HOME_DIR}/Projects`,
  })
  .option("-o, --open-vs-code", "Open the repository in VS Code.", {
    default: true,
  })
  .option("--dry-run", "Print the command that would be run.")
  .option("--print-cd", "Print a shell-friendly 'code ... && cd ...' command for eval.", {
    default: false,
  })
  .action(cloneAction);
