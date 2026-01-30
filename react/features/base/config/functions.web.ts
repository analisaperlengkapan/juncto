import { IReduxState } from '../../app/types';
import JunctoMeetJS from '../../base/lib-juncto';

import {
    IConfig,
    IDeeplinkingConfig,
    IDeeplinkingDesktopConfig,
    IDeeplinkingMobileConfig
} from './configType';

export * from './functions.any';

/**
 * Removes all analytics related options from the given configuration, in case of a libre build.
 *
 * @param {*} _config - The configuration which needs to be cleaned up.
 * @returns {void}
 */
export function _cleanupConfig(_config: IConfig) {
    return;
}

/**
 * Returns the replaceParticipant config.
 *
 * @param {Object} state - The state of the app.
 * @returns {boolean}
 */
export function getReplaceParticipant(state: IReduxState): string | undefined {
    return state['features/base/config'].replaceParticipant;
}

/**
 * Returns the configuration value of web-hid feature.
 *
 * @param {Object} state - The state of the app.
 * @returns {boolean} True if web-hid feature should be enabled, otherwise false.
 */
export function getWebHIDFeatureConfig(state: IReduxState): boolean {
    return state['features/base/config'].enableWebHIDFeature || false;
}

/**
 * Returns whether audio level measurement is enabled or not.
 *
 * @param {Object} state - The state of the app.
 * @returns {boolean}
 */
export function areAudioLevelsEnabled(state: IReduxState): boolean {
    return !state['features/base/config'].disableAudioLevels && JunctoMeetJS.isCollectingLocalStats();
}

/**
 * Sets the defaults for deeplinking.
 *
 * @param {IDeeplinkingConfig} deeplinking - The deeplinking config.
 * @returns {void}
 */
export function _setDeeplinkingDefaults(deeplinking: IDeeplinkingConfig) {
    deeplinking.desktop = deeplinking.desktop || {} as IDeeplinkingDesktopConfig;
    deeplinking.android = deeplinking.android || {} as IDeeplinkingMobileConfig;
    deeplinking.ios = deeplinking.ios || {} as IDeeplinkingMobileConfig;

    const { android, desktop, ios } = deeplinking;

    desktop.appName = desktop.appName || 'Juncto';
    desktop.appScheme = desktop.appScheme || 'juncto';
    desktop.download = desktop.download || {};
    desktop.download.windows = desktop.download.windows
        || 'https://github.com/juncto/juncto-electron/releases/latest/download/juncto.exe';
    desktop.download.macos = desktop.download.macos
        || 'https://github.com/juncto/juncto-electron/releases/latest/download/juncto.dmg';
    desktop.download.linux = desktop.download.linux
        || 'https://github.com/juncto/juncto-electron/releases/latest/download/juncto-x86_64.AppImage';

    ios.appName = ios.appName || 'Juncto';
    ios.appScheme = ios.appScheme || 'org.juncto.meet';
    ios.downloadLink = ios.downloadLink
        || 'https://itunes.apple.com/us/app/juncto/id1165103905';

    android.appName = android.appName || 'Juncto';
    android.appScheme = android.appScheme || 'org.juncto.meet';
    android.downloadLink = android.downloadLink
        || 'https://play.google.com/store/apps/details?id=org.juncto.meet';
    android.appPackage = android.appPackage || 'org.juncto.meet';
    android.fDroidUrl = android.fDroidUrl || 'https://f-droid.org/packages/org.juncto.meet/';
}
