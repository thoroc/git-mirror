#!/usr/bin/env -S deno run --allow-env --allow-read --allow-write --allow-run
import { Command } from 'jsr:@cliffy/command@^1.0.0-rc.7';
import { Confirm } from 'jsr:@cliffy/prompt@1.0.0-rc.7/confirm';
import { exists } from 'jsr:@std/fs';
import chalk from 'npm:chalk';
import { HOME_DIR } from './utils/constants.ts';
import {
  cloneRepo,
  fetchRepo,
  findExecutable,
  getLocalPath,
  runExecutable,
} from './utils/mod.ts';

interface CloneOptions {
  openVsCode?: boolean;
  rootDir?: string;
}

const cloneAction = async (options: CloneOptions, repo: string) => {
  const localRepo = getLocalPath(
    repo,
    options.rootDir ?? `${HOME_DIR}/Projects`
  );
  console.log(
    `Cloning repository: ${chalk.green(repo)} to ${chalk.green(localRepo)}`
  );

  const dirAlreadyExists = await exists(localRepo);
  if (dirAlreadyExists) {
    await fetchRepo(localRepo);
  } else {
    await cloneRepo(repo, localRepo);
  }

  const openVsCode =
    options.openVsCode ??
    (await Confirm.prompt('Open the repository in VS Code?'));

  if (openVsCode) {
    const vscode = await findExecutable('code');
    runExecutable(vscode, [localRepo]);
  }

  console.log(`Move to the project's directory: cd ${chalk.green(localRepo)}`);
};

await new Command()
  .name('clone')
  .version('0.1.1')
  .description('Clone a Git repository into the ~/Projects directory.')
  .arguments('<repo:string>')
  .option('-r, --root <rootDir>', 'The root directory.', {
    default: `${HOME_DIR}/Projects`,
  })
  .option('-o, --open-vs-code', 'Open the repository in VS Code.', {
    default: false,
  })
  .action(cloneAction)
  .parse(Deno.args);
