export const printVersion = async () => {
  const file = Bun.file("./package.json");
  const contents = await file.json();
  console.log("Version:", `v${contents.version}`);
};
