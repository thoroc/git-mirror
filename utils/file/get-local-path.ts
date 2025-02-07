import { getDirPathFromRepo } from './get-dir-path-from-repo.ts';
import { getHostFromRepo } from './get-host-from-repo.ts';

export const getLocalPath = (repo: string, targetBaseDir: string): string => {
  const host = getHostFromRepo(repo);
  const dirPath = getDirPathFromRepo(repo);
  const targetPath = `${targetBaseDir}/${host}/${dirPath}`;

  // find all sequential directories that exist
  const dirs = targetPath.split('/');
  const remainingPath: string[] = [];

  for (const dir of dirs) {
    if (!remainingPath.includes(dir)) {
      remainingPath.push(dir);
    }
  }

  return remainingPath.join('/');
};
