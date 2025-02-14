import { colors } from "jsr:@cliffy/ansi@^1.0.0-rc.7/colors";

export const getHostFromRepo = (repo: string): string => {
  let host: string;

  if (repo.startsWith("git@")) {
    const repoParts = repo.split(":");
    host = repoParts[0].split("@")[1].split(".")[0];
  } else if (repo.startsWith("https://")) {
    const repoParts = repo.split("/");
    host = repoParts[2].split(".")[0];
  } else {
    console.error(colors.bgBrightRed(`Invalid Git repository URL: ${repo}`));
    Deno.exit(1);
  }

  return host;
};
