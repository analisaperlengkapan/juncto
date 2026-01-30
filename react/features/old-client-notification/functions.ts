import { browser } from '../base/lib-juncto';

/**
 * Returns true if Juncto is running in too old juncto-electron app and false otherwise.
 *
 * @returns {boolean} - True if Juncto is running in too old juncto-electron app and false otherwise.
 */
export function isOldJunctoMeetElectronApp() {
    if (!browser.isElectron()) {
        return false;
    }

    // @ts-ignore
    const match = navigator.userAgent.match(/(JunctoMeet)\s*\/\s*((\d+)\.[^\s]*)/);

    if (!Array.isArray(match) || match.length < 3) {
        return false;
    }

    const majorVersion = Number(match[3]);

    if (isNaN(majorVersion) || majorVersion >= 2022) {
        return false;
    }

    return true;
}
