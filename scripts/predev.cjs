if (process.env.ZSTIO_TV) process.exit(0);

const rl = require("readline").createInterface({
  input: process.stdin,
  output: process.stdout,
});

const gray = "\x1b[47;30m";
const reset = "\x1b[0m";

process.stdout.write(
  `\n You sure you are running zstio-tv correctly? use ${gray} make dev ${reset} to run the whole application\n\nY/N: `
);

rl.question("", (answer) => {
  rl.close();
  if (answer.toLowerCase() !== "y") process.exit(1);
});
