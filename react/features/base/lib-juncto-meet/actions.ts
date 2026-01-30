// @ts-expect-error
import { junctoLocalStorage } from '@juncto/js-utils';

import { IStore } from '../../app/types';
import { isOnline } from '../net-info/selectors';

import JunctoMeetJS from './_';
import {
    LIB_DID_DISPOSE,
    LIB_DID_INIT,
    LIB_INIT_ERROR,
    LIB_WILL_DISPOSE,
    LIB_WILL_INIT
} from './actionTypes';
import { isAnalyticsEnabled } from './functions.any';
import logger from './logger';

/**
 * Disposes (of) lib-juncto.
 *
 * @returns {Function}
 */
export function disposeLib() {
    return (dispatch: IStore['dispatch']) => {
        dispatch({ type: LIB_WILL_DISPOSE });

        // TODO Currently, lib-juncto doesn't have the functionality to
        // dispose itself.
        dispatch({ type: LIB_DID_DISPOSE });
    };
}

/**
 * Initializes lib-juncto (i.e. {@link invokes JunctoMeetJS.init()}) with the
 * current config(uration).
 *
 * @returns {Function}
 */
export function initLib() {
    return (dispatch: IStore['dispatch'], getState: IStore['getState']) => {
        const state = getState();
        const config = state['features/base/config'];

        if (!config) {
            throw new Error('Cannot init lib-juncto without config');
        }

        dispatch({ type: LIB_WILL_INIT });

        try {
            JunctoMeetJS.init({
                enableAnalyticsLogging: isAnalyticsEnabled(getState),
                ...config,
                externalStorage: junctoLocalStorage.isLocalStorageDisabled() ? junctoLocalStorage : undefined
            });
            JunctoMeetJS.setNetworkInfo({
                isOnline: isOnline(state)
            });

            logger.info(`lib-juncto version: ${JunctoMeetJS.version}`);
            logger.info(`User Agent: ${navigator.userAgent}`);

            dispatch({ type: LIB_DID_INIT });
        } catch (error: any) {
            dispatch(libInitError(error));
        }
    };
}

/**
 * Notifies about a specific error raised by {@link JunctoMeetJS.init()}.
 *
 * @param {Error} error - The Error raised by JunctoMeetJS.init().
 * @returns {{
 *     type: LIB_INIT_ERROR,
 *     error: Error
 * }}
 */
export function libInitError(error: Error) {
    return {
        type: LIB_INIT_ERROR,
        error
    };
}
