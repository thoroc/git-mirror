import { HOME_DIR } from "@scope/env";
import { findExecutable, runExecutable } from "@scope/exec";
import { getLocalPath } from "@scope/fs";
import { cloneRepo, fetchRepo } from "@scope/git";
import { colors } from "jsr:@cliffy/ansi@^1.0.0-rc.7/colors";
import { Confirm } from "jsr:@cliffy/prompt@1.0.0-rc.7/confirm";
import { exists } from "jsr:@std/fs";

interface CloneOptions {
  openVsCode?: boolean;
  rootDir?: string;
  dryRun?: boolean;
}

export const cloneAction = async (options: CloneOptions, repo: string) => {
  if (options.dryRun) {
    console.log(
      colors.bgYellow(
        "Dry run mode ... none of the commands will actually be run.",
      ),
    );
  }

  const localRepo = getLocalPath(
    repo,
    options.rootDir ?? `${HOME_DIR}/Projects`,
  );
  console.log(
    `Cloning repository: ${colors.green(repo)} to ${colors.green(localRepo)}`,
  );

  const dirAlreadyExists = await exists(localRepo);
  if (dirAlreadyExists) {
    if (options.dryRun) {
      console.log(
        colors.yellow(`> Dry run: Fetching repository: ${localRepo}`),
      );
    } else {
      await fetchRepo(localRepo);
    }
  } else {
    if (options.dryRun) {
      console.log(colors.yellow(`> Dry run: Cloning repository: ${repo}`));
    } else {
      await cloneRepo(repo, localRepo);
    }
  }

  const openVsCode = options.openVsCode ??
    (await Confirm.prompt({
      message: "Open the repository in VS Code?",
      default: true,
    }));

  if (openVsCode) {
    if (options.dryRun) {
      console.log(
        colors.yellow(`> Dry run: Opening repository in VS Code: ${localRepo}`),
      );
    } else {
      const vscode = await findExecutable("code");
      runExecutable(vscode, [localRepo]);
    }
  }

  console.log(
    `To move to the project's directory, please run: "cd ${
      colors.green(
        localRepo,
      )
    }"`,
  );
};
