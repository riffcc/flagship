import type {Session} from 'electron';
import {app, shell} from 'electron'; // `session` is not directly imported but accessed via `contents.session`
import {URL} from 'url';

type Permission = Parameters<
  Exclude<Parameters<Session['setPermissionRequestHandler']>[0], null>
>[1];

/**
 * A list of origins that you allow open INSIDE the application and permissions for them.
 *
 * In development mode you need allow open `VITE_DEV_SERVER_URL`.
 */
const permissions = new Set<Permission>(['clipboard-sanitized-write']);
const ALLOWED_ORIGINS_AND_PERMISSIONS = new Map<string, Set<Permission>>(
  import.meta.env.DEV && import.meta.env.VITE_DEV_SERVER_URL
    ? [[new URL(import.meta.env.VITE_DEV_SERVER_URL).origin, permissions]]
    : [['null', permissions]],
);

/**
 * A list of origins that you allow open IN BROWSER.
 * Navigation to the origins below is only possible if the link opens in a new window.
 *
 * @example
 * <a
 *   target="_blank"
 *   href="https://github.com/"
 * >
 */
const ALLOWED_EXTERNAL_ORIGINS = new Set<`https://${string}`>([
  'https://github.com',
  'https://docu.xn--rseau-constellation-bzb.ca',
  'https://matrix.to',
]);

app.on('web-contents-created', (_, contents) => {
  /**
   * Block navigation to origins not on the allowlist.
   *
   * Navigation exploits are quite common. If an attacker can convince the app to navigate away from its current page,
   * they can possibly force the app to open arbitrary web resources/websites on the web.
   *
   * @see https://www.electronjs.org/docs/latest/tutorial/security#13-disable-or-limit-navigation
   */
  contents.on('will-navigate', (event, url) => {
    const {origin} = new URL(url);
    if (ALLOWED_ORIGINS_AND_PERMISSIONS.has(origin)) {
      return;
    }

    // Prevent navigation
    event.preventDefault();

    if (import.meta.env.DEV) {
      console.warn(`Blocked navigating to disallowed origin: ${origin}`);
    }
  });

  /**
   * Block requests for disallowed permissions.
   * By default, Electron will automatically approve all permission requests.
   *
   * @see https://www.electronjs.org/docs/latest/tutorial/security#5-handle-session-permission-requests-from-remote-content
   */
  contents.session.setPermissionRequestHandler((webContents, permission, callback) => {
    const {origin} = new URL(webContents.getURL());

    const permissionGranted = !!ALLOWED_ORIGINS_AND_PERMISSIONS.get(origin)?.has(permission);
    callback(permissionGranted);

    if (!permissionGranted && import.meta.env.DEV) {
      console.warn(`${origin} requested permission for '${permission}', but was rejected.`);
    }
  });

  /**
   * Hyperlinks leading to allowed sites are opened in the default browser.
   *
   * The creation of new `webContents` is a common attack vector. Attackers attempt to convince the app to create new windows,
   * frames, or other renderer processes with more privileges than they had before; or with pages opened that they couldn't open before.
   * You should deny any unexpected window creation.
   *
   * @see https://www.electronjs.org/docs/latest/tutorial/security#14-disable-or-limit-creation-of-new-windows
   * @see https://www.electronjs.org/docs/latest/tutorial/security#15-do-not-use-openexternal-with-untrusted-content
   */
  contents.setWindowOpenHandler(({url}) => {
    const {origin, protocol} = new URL(url);

    // @ts-expect-error Type checking is performed in runtime.
    if (protocol === 'mailto:' || ALLOWED_EXTERNAL_ORIGINS.has(origin)) {
      // Open url in default browser.
      shell.openExternal(url).catch(console.error);
    } else if (import.meta.env.DEV) {
      console.warn(`Blocked the opening of a disallowed origin: ${origin}`);
    }

    // Prevent creating a new window.
    return {action: 'deny'};
  });

  /**
   * Verify webview options before creation.
   *
   * Strip away preload scripts, disable Node.js integration, and ensure origins are on the allowlist.
   *
   * @see https://www.electronjs.org/docs/latest/tutorial/security#12-verify-webview-options-before-creation
   */
  contents.on('will-attach-webview', (event, webPreferences, params) => {
    const {origin} = new URL(params.src);
    if (!ALLOWED_ORIGINS_AND_PERMISSIONS.has(origin)) {
      if (import.meta.env.DEV) {
        console.warn(`A webview tried to attach ${params.src}, but was blocked.`);
      }

      event.preventDefault();
      return;
    }

    // Strip away preload scripts if unused or verify their location is legitimate.
    delete webPreferences.preload;
    // @ts-expect-error `preloadURL` exists. - @see https://www.electronjs.org/docs/latest/api/web-contents#event-will-attach-webview
    delete webPreferences.preloadURL;

    // Disable Node.js integration
    webPreferences.nodeIntegration = false;

    // Enable contextIsolation
    webPreferences.contextIsolation = true;
  });

  /**
   * Modify Content-Security-Policy to allow 'unsafe-eval' for script-src.
   * This is needed for Playwright's waitForFunction and potentially other dev/test tools.
   * It also ensures a baseline object-src for security.
   */
  contents.session.webRequest.onHeadersReceived((details, callback) => {
    if (details.responseHeaders && (details.resourceType === 'mainFrame' || details.resourceType === 'subFrame')) {
      console.log(`[CSP Modifier] Intercepting headers for URL: ${details.url}, ResourceType: ${details.resourceType}`);
      const originalCspHeader = details.responseHeaders['content-security-policy'] || details.responseHeaders['Content-Security-Policy'];
      console.log('[CSP Modifier] Original CSP Header from details:', originalCspHeader);

      const cspHeaderKeys = Object.keys(details.responseHeaders).filter(
        key => key.toLowerCase() === 'content-security-policy',
      );

      let currentCsp = '';
      if (cspHeaderKeys.length > 0 && details.responseHeaders[cspHeaderKeys[0]]) {
        const cspValue = details.responseHeaders[cspHeaderKeys[0]];
        currentCsp = Array.isArray(cspValue) ? cspValue.join(';') : cspValue;
      }

      let newCspDirectives = [];
      if (currentCsp) {
        newCspDirectives = currentCsp.split(';').map(d => d.trim()).filter(d => d);
      }

      let scriptSrcFound = false;
      for (let i = 0; i < newCspDirectives.length; i++) {
        if (newCspDirectives[i].toLowerCase().startsWith('script-src')) {
          scriptSrcFound = true;
          if (!newCspDirectives[i].includes("'unsafe-eval'")) {
            newCspDirectives[i] += " 'unsafe-eval'";
          }
          break;
        }
      }

      if (!scriptSrcFound) {
        // Based on the error message, 'self' and 'blob:' are already part of the effective CSP.
        // Add 'unsafe-eval' to it.
        newCspDirectives.push("script-src 'self' blob: 'unsafe-eval'");
      }

      let objectSrcFound = false;
      for (let i = 0; i < newCspDirectives.length; i++) {
        if (newCspDirectives[i].toLowerCase().startsWith('object-src')) {
          objectSrcFound = true;
          // Example: ensure it's restrictive like 'self' or 'none'
          // For now, just ensuring it exists if we add it.
          if (!newCspDirectives[i].includes("'self'")) {
            // This logic could be more nuanced if object-src exists but is too permissive.
            // For simplicity, if it exists, we leave it; if not, we add a restrictive one.
          }
          break;
        }
      }
      if (!objectSrcFound) {
        newCspDirectives.push("object-src 'self'");
      }
      
      let baseUriFound = false;
      for (let i = 0; i < newCspDirectives.length; i++) {
        if (newCspDirectives[i].toLowerCase().startsWith('base-uri')) {
          baseUriFound = true;
          break;
        }
      }
      if(!baseUriFound) {
        newCspDirectives.push("base-uri 'self'");
      }


      const newCspString = newCspDirectives.join('; ');

      // Remove all existing CSP headers (case-insensitive)
      Object.keys(details.responseHeaders).forEach(key => {
        if (key.toLowerCase() === 'content-security-policy') {
          delete details.responseHeaders![key];
        }
      });
      // Set the new CSP header
      details.responseHeaders['Content-Security-Policy'] = [newCspString];
      console.log('[CSP Modifier] New CSP Header Set:', newCspString);
    }
    callback({responseHeaders: details.responseHeaders});
  });
});
