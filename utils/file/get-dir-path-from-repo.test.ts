import { assertEquals } from 'jsr:@std/assert';
import { describe, it } from 'jsr:@std/testing/bdd';
import { getDirPathFromRepo } from './get-dir-path-from-repo.ts';

describe('getDirPathFromRepo', () => {
  const TEST_CASES = [
    {
      repo: 'git@git-host.com:organisation/project.git',
      expected: 'organisation/project',
    },
    {
      repo: 'https://git-host.com/organisation/project.git',
      expected: 'organisation/project',
    },
  ];

  for (const { repo, expected } of TEST_CASES) {
    it(`should return [${expected}] from a Git repository URL [${repo}]`, () => {
      assertEquals(getDirPathFromRepo(repo), expected);
    });
  }
});
