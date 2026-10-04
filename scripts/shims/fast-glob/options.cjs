// The fast-glob options this stand-in translates. tryscript passes `ignore`, `absolute`,
// and `dot`; the others share their meaning and default with tinyglobby. Anything else
// throws instead of being dropped, so a tryscript upgrade that starts relying on another
// fast-glob behavior fails loudly here rather than matching a different file set.
const SUPPORTED = new Set(["absolute", "cwd", "dot", "ignore", "onlyFiles"]);

function translate(options = {}) {
  for (const key of Object.keys(options)) {
    if (!SUPPORTED.has(key)) {
      throw new Error(`fast-glob stand-in: option "${key}" is not translated; see scripts/shims/fast-glob/README.md`);
    }
  }
  // fast-glob never expands a directory pattern into its contents; tinyglobby does by default.
  return { ...options, expandDirectories: false };
}

module.exports = { translate };
