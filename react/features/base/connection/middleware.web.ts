import MiddlewareRegistry from '../redux/MiddlewareRegistry';

import { CONNECTION_WILL_CONNECT } from './actionTypes';

/**
 * The feature announced so we can distinguish junbri participants.
 *
 * @type {string}
 */
export const DISCO_JIBRI_FEATURE = 'http://juncto.org/protocol/junbri';

MiddlewareRegistry.register(({ getState }) => next => action => {
    switch (action.type) {
    case CONNECTION_WILL_CONNECT: {
        const { connection } = action;
        const { iAmRecorder } = getState()['features/base/config'];

        if (iAmRecorder) {
            connection.addFeature(DISCO_JIBRI_FEATURE);
        }

        // @ts-ignore
        APP.connection = connection;

        break;
    }
    }

    return next(action);
});
