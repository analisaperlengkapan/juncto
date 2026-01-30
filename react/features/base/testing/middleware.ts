import { IStore } from '../../app/types';
import { CONFERENCE_JOIN_IN_PROGRESS } from '../conference/actionTypes';
import { IJunctoConference } from '../conference/reducer';
import { SET_CONFIG } from '../config/actionTypes';
import { JunctoConferenceEvents } from '../lib-juncto';
import MiddlewareRegistry from '../redux/MiddlewareRegistry';
import { getJunctoMeetGlobalNS } from '../util/helpers';

import { setConnectionState } from './actions';
import {
    audioMute,
    audioUnmute,
    getLocalCameraEncoding,
    getRemoteVideoType,
    isLargeVideoReceived,
    isRemoteVideoReceived,
    isTestModeEnabled,
    videoMute,
    videoUnmute
} from './functions';
import logger from './logger';

/**
 * The Redux middleware of the feature testing.
 *
 * @param {Store} store - The Redux store.
 * @returns {Function}
 * @private
 */
MiddlewareRegistry.register(store => next => action => {
    switch (action.type) {
    case CONFERENCE_JOIN_IN_PROGRESS:
        _bindConferenceConnectionListener(action.conference, store);
        break;
    case SET_CONFIG: {
        const result = next(action);

        _bindTortureHelpers(store);

        return result;
    }
    }

    return next(action);
});

/**
 * Binds a handler which will listen for the connection related conference
 * events (in the lib-juncto internals those are associated with the ICE
 * connection state).
 *
 * @param {JunctoConference} conference - The {@link JunctoConference} for which
 * the conference will join even is dispatched.
 * @param {Store} store - The redux store in which the specified action is being
 * dispatched.
 * @private
 * @returns {void}
 */
function _bindConferenceConnectionListener(conference: IJunctoConference, { dispatch }: IStore) {

    conference.on(
        JunctoConferenceEvents.CONNECTION_ESTABLISHED,
        _onConnectionEvent.bind(
            null, JunctoConferenceEvents.CONNECTION_ESTABLISHED, dispatch));
    conference.on(
        JunctoConferenceEvents.CONNECTION_RESTORED,
        _onConnectionEvent.bind(
            null, JunctoConferenceEvents.CONNECTION_RESTORED, dispatch));
    conference.on(
        JunctoConferenceEvents.CONNECTION_INTERRUPTED,
        _onConnectionEvent.bind(
            null, JunctoConferenceEvents.CONNECTION_INTERRUPTED, dispatch));
}

/**
 * Binds all the helper functions needed by torture.
 *
 * @param {IStore} store - The redux store.
 * @private
 * @returns {void}
 */
function _bindTortureHelpers(store: IStore) {
    const { getState } = store;

    // We bind helpers only if testing mode is enabled
    if (!isTestModeEnabled(getState())) {
        return;
    }

    // All torture helper methods go in here
    getJunctoMeetGlobalNS().testing = {
        audioMute: audioMute.bind(null, store),
        audioUnmute: audioUnmute.bind(null, store),
        getRemoteVideoType: getRemoteVideoType.bind(null, store),
        isLargeVideoReceived: isLargeVideoReceived.bind(null, store),
        getLocalCameraEncoding: getLocalCameraEncoding.bind(null, store),
        isRemoteVideoReceived: isRemoteVideoReceived.bind(null, store),
        videoMute: videoMute.bind(null, store),
        videoUnmute: videoUnmute.bind(null, store),
    };
}

/**
 * The handler function for conference connection events which will store the
 * latest even name in the Redux store of feature testing.
 *
 * @param {string} event - One of the lib-juncto JunctoConferenceEvents.
 * @param {Function} dispatch - The dispatch function of the current Redux
 * store.
 * @returns {void}
 * @private
 */
function _onConnectionEvent(event: string, dispatch: IStore['dispatch']) {
    switch (event) {
    case JunctoConferenceEvents.CONNECTION_ESTABLISHED:
    case JunctoConferenceEvents.CONNECTION_INTERRUPTED:
    case JunctoConferenceEvents.CONNECTION_RESTORED:
        dispatch(setConnectionState(event));
        break;
    default:
        logger.error(`onConnectionEvent - unsupported event type: ${event}`);
        break;
    }
}

