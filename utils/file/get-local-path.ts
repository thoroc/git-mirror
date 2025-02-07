import { getDirPathFromRepo } from './get-dir-path-from-repo.ts';
import { getHostFromRepo } from './get-host-from-repo.ts';

export const getLocalPath = (repo: string, targetBaseDir: string): string => {
  const host = getHostFromRepo(repo);
  const dirPath = getDirPathFromRepo(repo);
  const targetPath = `${targetBaseDir}/${host}/${dirPath}`;

  // find all sequential directories that exist
  const dirs = targetPath.split('/');
  const remainingPath: string[] = [];

  for (let i = 0; i < dirs.length; i++) {
    if (
      !remainingPath.includes(dirs[i]) ||
      (dirs[i] === dirs[i - 1] && i === dirs.length - 1)
    ) {
      remainingPath.push(dirs[i]);
    }
  }

  return remainingPath.join('/');
};
