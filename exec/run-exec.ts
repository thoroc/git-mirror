import { colors } from "jsr:@cliffy/ansi@^1.0.0-rc.7/colors";

export const runExecutable = async (
  executable: string,
  args: string[],
): Promise<Deno.CommandStatus> => {
  try {
    const command = await new Deno.Command(executable, {
      args,
      stdin: "piped",
    });
    const child = await command.spawn();
    const status = await child.status;

    return status;
  } catch (error) {
    console.error(colors.bgBrightRed(`Error running executable: ${error}`));
    Deno.exit(1);
  }
};
