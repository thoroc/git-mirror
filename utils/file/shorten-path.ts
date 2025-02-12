interface ShortenOptions {
  separator?: string;
  min?: number;
  left?: number;
  right?: number;
}

export const shortenPath = (path: string, options?: ShortenOptions): string => {
  const min = options?.min ?? 3;
  const separator = options?.separator ?? "/";
  const parts = path.split(separator);

  if (parts.length <= min) {
    return path;
  }

  const left = options?.left ?? 1;
  const first = parts.slice(0, left + 1).join("/");

  const right = options?.right ?? 2;
  const last = parts.slice(-right).join("/");

  return `${first}/.../${last}`;
};
