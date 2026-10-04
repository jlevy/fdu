const { glob, globSync } = require("tinyglobby");
const { translate } = require("./options.cjs");

function fastGlob(patterns, options) {
  return glob(patterns, translate(options));
}
fastGlob.glob = fastGlob;
fastGlob.async = fastGlob;
fastGlob.sync = (patterns, options) => globSync(patterns, translate(options));
fastGlob.globSync = fastGlob.sync;

module.exports = fastGlob;
