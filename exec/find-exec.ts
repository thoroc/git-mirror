import { colors } from "jsr:@cliffy/ansi@^1.0.0-rc.7/colors";

export const findExecutable = async (executable: string): Promise<string> => {
  try {
    const command = await new Deno.Command("which", {
      args: [executable],
      stdout: "piped",
    });
    const { stdout } = await command.output();

    return new TextDecoder().decode(stdout).trim();
  } catch (error) {
    console.error(colors.bgBrightRed(`Error finding git path: ${error}`));
    Deno.exit(1);
  }
};
