import { colors } from "jsr:@cliffy/ansi@^1.0.0-rc.7/colors";

export const getDirPathFromRepo = (repo: string): string => {
  let dirPath: string;

  if (repo.startsWith("git+https://")) {
    const repoParts = repo.split("/");
    dirPath = repoParts.slice(3).join("/").replace(".git", "");
  } else if (repo.startsWith("git@")) {
    const repoParts = repo.split(":");
    dirPath = repoParts[1].replace(".git", "");
  } else if (repo.startsWith("https://")) {
    const repoParts = repo.split("/");
    dirPath = repoParts.slice(3).join("/").replace(".git", "");
  } else {
    console.error(colors.bgBrightRed(`Invalid Git repository URL: ${repo}`));
    Deno.exit(1);
  }

  return dirPath;
};
