import { IStateful } from '../app/types';
import { ConnectionFailedError } from '../connection/types';
import { toState } from '../redux/functions';

import JunctoMeetJS from './_';


const JunctoConferenceErrors = JunctoMeetJS.errors.conference;
const JunctoConnectionErrors = JunctoMeetJS.errors.connection;

/**
 * Creates a {@link JunctoLocalTrack} model from the given device id.
 *
 * @param {string} type - The media type of track being created. Expected values
 * are "video" or "audio".
 * @param {string} deviceId - The id of the target media source.
 * @param {number} [timeout] - A timeout for the JunctoMeetJS.createLocalTracks function call.
 * @param {Object} additionalOptions - Extra options to be passed to lib-juncto's {@code createLocalTracks}.
 *
 * @returns {Promise<JunctoLocalTrack>}
 */
export function createLocalTrack(type: string, deviceId: string | null, timeout?: number | null,
        additionalOptions?: Object) {
    return (
        JunctoMeetJS.createLocalTracks({
            cameraDeviceId: deviceId,
            devices: [ type ],
            micDeviceId: deviceId,
            timeout,
            ...additionalOptions
        })
            .then(([ junctoLocalTrack ]: any[]) => junctoLocalTrack));
}

/**
 * Determines whether analytics is enabled in a specific redux {@code store}.
 *
 * @param {IStateful} stateful - The redux store, state, or
 * {@code getState} function.
 * @returns {boolean} If analytics is enabled, {@code true}; {@code false},
 * otherwise.
 */
export function isAnalyticsEnabled(stateful: IStateful) {
    const { disableThirdPartyRequests, analytics = {} } = toState(stateful)['features/base/config'];

    return !(disableThirdPartyRequests || analytics.disabled);
}

/**
 * Determines whether a specific {@link JunctoConferenceErrors} instance
 * indicates a fatal {@link JunctoConference} error.
 *
 * FIXME Figure out the category of errors defined by the function and describe
 * that category. I've currently named the category fatal because it appears to
 * be used in the cases of unrecoverable errors that necessitate a reload.
 *
 * @param {Error|string} error - The {@code JunctoConferenceErrors} instance to
 * categorize/classify or an {@link Error}-like object.
 * @returns {boolean} If the specified {@code JunctoConferenceErrors} instance
 * indicates a fatal {@code JunctoConference} error, {@code true}; otherwise,
 * {@code false}.
 */
export function isFatalJunctoConferenceError(error: Error | string) {
    if (typeof error !== 'string') {
        error = error.name; // eslint-disable-line no-param-reassign
    }

    return (
        error === JunctoConferenceErrors.FOCUS_DISCONNECTED
            || error === JunctoConferenceErrors.FOCUS_LEFT
            || error === JunctoConferenceErrors.ICE_FAILED
            || error === JunctoConferenceErrors.OFFER_ANSWER_FAILED
            || error === JunctoConferenceErrors.VIDEOBRIDGE_NOT_AVAILABLE);
}

/**
 * Determines whether a specific {@link JunctoConnectionErrors} instance
 * indicates a fatal {@link JunctoConnection} error.
 *
 * FIXME Figure out the category of errors defined by the function and describe
 * that category. I've currently named the category fatal because it appears to
 * be used in the cases of unrecoverable errors that necessitate a reload.
 *
 * @param {Error|string} error - The {@code JunctoConnectionErrors} instance to
 * categorize/classify or an {@link Error}-like object.
 * @returns {boolean} If the specified {@code JunctoConnectionErrors} instance
 * indicates a fatal {@code JunctoConnection} error, {@code true}; otherwise,
 * {@code false}.
 */
export function isFatalJunctoConnectionError(error: Error | string | ConnectionFailedError) {
    if (typeof error !== 'string') {
        error = error.name; // eslint-disable-line no-param-reassign
    }

    return (
        error === JunctoConnectionErrors.CONNECTION_DROPPED_ERROR
            || error === JunctoConnectionErrors.OTHER_ERROR
            || error === JunctoConnectionErrors.SERVER_ERROR);
}
