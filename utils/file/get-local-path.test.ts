import { assertEquals } from 'jsr:@std/assert';
import { describe, it } from 'jsr:@std/testing/bdd';
import { getLocalPath } from './get-local-path.ts';
import { shortenPath } from './shorten-path.ts';

describe('getLocalPath', () => {
  const TEST_CASES = [
    {
      targetBaseDir: '/home/user/Projects',
      repo: 'https://git-host.com/project/project.git',
      expected: '/home/user/Projects/git-host/project/project',
    },
    {
      targetBaseDir: '/home/user/Projects',
      repo: 'https://git-host.com/organisation/project.git',
      expected: '/home/user/Projects/git-host/organisation/project',
    },
    {
      targetBaseDir: '/home/user/Projects/git-host',
      repo: 'https://git-host.com/organisation/project.git',
      expected: '/home/user/Projects/git-host/organisation/project',
    },
    {
      targetBaseDir: '/home/user/Projects',
      repo: 'https://git-host.com/organisation/project.git',
      expected: '/home/user/Projects/git-host/organisation/project',
    },
    {
      targetBaseDir: '/home/user/Projects/git-host',
      repo: 'https://git-host.com/organisation/project.git',
      expected: '/home/user/Projects/git-host/organisation/project',
    },
    {
      targetBaseDir: '/home/user/Projects/git-host/organisation',
      repo: 'https://git-host.com/organisation/project.git',
      expected: '/home/user/Projects/git-host/organisation/project',
    },
    {
      targetBaseDir: '/home/user/Projects/git-host/organisation/project',
      repo: 'https://git-host.com/organisation/project.git',
      expected: '/home/user/Projects/git-host/organisation/project',
    },
  ];

  for (const { targetBaseDir, repo, expected } of TEST_CASES) {
    const shortenExpected = shortenPath(expected);
    const shortenRepo = shortenPath(repo, { right: 1 });

    it(`should return [${shortenExpected}] from a Git repository URL [${shortenRepo}]`, () => {
      assertEquals(getLocalPath(repo, targetBaseDir), expected);
    });
  }
});
