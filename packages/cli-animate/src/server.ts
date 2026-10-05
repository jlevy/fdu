/**
 * A local HTTP server for the stage page, used by `render` (headless capture) and `serve`
 * (a person watching). It serves only the stage page, the pinned fonts, xterm.js from the
 * installed package, and one cast at /recording.cast.
 */

import { readFile } from 'node:fs/promises';
import { createServer, type Server } from 'node:http';
import { createRequire } from 'node:module';
import { dirname, extname, join, normalize, sep } from 'node:path';
import { FONTS_DIR, STAGE_DIR } from './paths.js';

const resolvePackage = (name: string): string => dirname(createRequire(import.meta.url).resolve(`${name}/package.json`));
const XTERM_DIR = resolvePackage('@xterm/xterm');

const TYPES: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.woff2': 'font/woff2',
  '.cast': 'text/plain; charset=utf-8',
  '.json': 'application/json',
};

/** `relative` under `root`, or undefined if it would escape it. */
function within(root: string, relative: string): string | undefined {
  const path = normalize(join(root, relative));
  return path.startsWith(root.endsWith(sep) ? root : root + sep) ? path : undefined;
}

function resolveRoute(pathname: string, castFile: string): string | undefined {
  if (pathname === '/' || pathname === '/stage.html') return join(STAGE_DIR, 'stage.html');
  if (pathname === '/recording.cast') return castFile;
  if (pathname.startsWith('/fonts/')) return within(FONTS_DIR, pathname.slice('/fonts/'.length));
  if (pathname.startsWith('/xterm/')) return within(XTERM_DIR, pathname.slice('/xterm/'.length));
  return undefined;
}

export interface StageServer {
  url: string;
  close(): Promise<void>;
}

export async function startStageServer(castFile: string, port = 0): Promise<StageServer> {
  const server: Server = createServer((request, response) => {
    const pathname = decodeURIComponent(new URL(request.url ?? '/', 'http://localhost').pathname);
    const file = resolveRoute(pathname, castFile);
    if (!file) {
      response.writeHead(404).end();
      return;
    }
    readFile(file).then(
      (body) => {
        response.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream', 'cache-control': 'no-store' });
        response.end(body);
      },
      () => response.writeHead(404).end(),
    );
  });
  await new Promise<void>((resolve) => server.listen(port, '127.0.0.1', resolve));
  const address = server.address();
  const actualPort = typeof address === 'object' && address ? address.port : port;
  return {
    url: `http://127.0.0.1:${actualPort}`,
    close: () => new Promise<void>((resolve) => server.close(() => resolve())),
  };
}
