import { assertEquals } from "jsr:@std/assert";
import { describe, it } from "jsr:@std/testing/bdd";
import { getHostFromRepo } from "./get-host-from-repo.ts";

describe("getHostFromRepo", () => {
  const TEST_CASES = [
    {
      repo: "git@git-host.com:organisation/project.git",
      expected: "git-host",
    },
    {
      repo: "https://git-host.com/organisation/project.git",
      expected: "git-host",
    },
    {
      repo: "git+https://git-host.com/organisation/project.git", 
      expected: "git-host",
    },
  ];

  for (const { repo, expected } of TEST_CASES) {
    it(`should return [${expected}] from a Git repository URL [${repo}]`, () => {
      assertEquals(getHostFromRepo(repo), expected);
    });
  }
});
