import type {Session} from 'electron';
import {app, shell} from 'electron';
import {URL} from 'url';

type Permission = Parameters<
  Exclude<Parameters<Session['setPermissionRequestHandler']>[0], null>
>[1];

const permissions = new Set<Permission>(['clipboard-sanitized-write']);
const ALLOWED_ORIGINS_AND_PERMISSIONS = new Map<string, Set<Permission>>(
  import.meta.env.DEV && import.meta.env.VITE_DEV_SERVER_URL
    ? [[new URL(import.meta.env.VITE_DEV_SERVER_URL).origin, permissions]]
    : [['null', permissions]],
);

const ALLOWED_EXTERNAL_ORIGINS = new Set<`https://${string}`>([
  'https://github.com',
]);

app.on('web-contents-created', (_, contents) => {
  contents.on('will-navigate', (event, url) => {
    const {origin} = new URL(url);
    if (ALLOWED_ORIGINS_AND_PERMISSIONS.has(origin)) {
      return;
    }
    event.preventDefault();
  });

  contents.session.setPermissionRequestHandler((webContents, permission, callback) => {
    const {origin} = new URL(webContents.getURL());
    const permissionGranted = !!ALLOWED_ORIGINS_AND_PERMISSIONS.get(origin)?.has(permission);
    callback(permissionGranted);
  });

  contents.setWindowOpenHandler(({url}) => {
    const {origin, protocol} = new URL(url);

    // @ts-expect-error Type checking is performed in runtime.
    if (protocol === 'mailto:' || ALLOWED_EXTERNAL_ORIGINS.has(origin)) {
      shell.openExternal(url).catch(console.error);
    }
    return {action: 'deny'};
  });

  contents.on('will-attach-webview', (event, webPreferences, params) => {
    const {origin} = new URL(params.src);
    if (!ALLOWED_ORIGINS_AND_PERMISSIONS.has(origin)) {
      event.preventDefault();
      return;
    }

    delete webPreferences.preload;
    // @ts-expect-error `preloadURL` exists.
    delete webPreferences.preloadURL;

    webPreferences.nodeIntegration = false;
    webPreferences.contextIsolation = true;
  });

  contents.session.webRequest.onHeadersReceived((details, callback) => {
    if (details.responseHeaders && (details.resourceType === 'mainFrame' || details.resourceType === 'subFrame')) {
      let newCspString: string;

      if (import.meta.env.PROD) {
        newCspString = "default-src 'self'; script-src 'self' blob: 'unsafe-eval'; object-src 'self'; base-uri 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self';";
      } else {
        newCspString = "default-src 'self'; script-src 'self' blob: 'unsafe-inline' 'unsafe-eval'; object-src 'self'; style-src 'self' 'unsafe-inline'; connect-src *;";
      }

      Object.keys(details.responseHeaders).forEach(key => {
        if (key.toLowerCase() === 'content-security-policy') {
          delete details.responseHeaders![key];
        }
      });
      details.responseHeaders['Content-Security-Policy'] = [newCspString];
    }
    callback({responseHeaders: details.responseHeaders});
  });
});
