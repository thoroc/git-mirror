export const runExecutable = async (
  executable: string,
  args: string[],
): Promise<Deno.CommandStatus> => {
  const command = await new Deno.Command(executable, {
    args,
    stdin: "piped",
  });
  const child = await command.spawn();
  const status = await child.status;

  return status;
};
