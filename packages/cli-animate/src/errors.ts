/** Exit codes: a check that ran and failed, versus input or environment the user must fix. */
export const EXIT_CHECK_FAILED = 1;
export const EXIT_USAGE = 2;

/** An error the command line reports as one line, with the exit code it chose. */
export class CliError extends Error {
  readonly exitCode: number;

  constructor(message: string, exitCode: number = EXIT_USAGE) {
    super(message);
    this.name = 'CliError';
    this.exitCode = exitCode;
  }
}
