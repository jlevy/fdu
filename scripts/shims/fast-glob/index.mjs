import { createRequire } from "node:module";

const fastGlob = createRequire(import.meta.url)("./index.cjs");

export default fastGlob;
export const { glob, async, sync, globSync } = fastGlob;
