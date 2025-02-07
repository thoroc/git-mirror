import chalk from 'npm:chalk';
import { findExecutable } from '../exec/find-exec.ts';

export const fetchRepo = async (localRepo: string): Promise<void> => {
  const git = await findExecutable('git');

  const pull = new Deno.Command(git, {
    cwd: localRepo,
    args: ['fetch', 'origin'],
    stdin: 'piped',
  });
  const child = await pull.spawn();
  const status = await child.status;

  if (!status.success) {
    console.error(chalk.bgRedBright(`Error fetching repository`));
  } else {
    console.log(chalk.bgGreenBright(`Repository fetching successfully`));
  }
};
