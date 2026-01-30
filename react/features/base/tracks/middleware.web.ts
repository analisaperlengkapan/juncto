import { AnyAction } from 'redux';

import { IStore } from '../../app/types';
import { hideNotification } from '../../notifications/actions';
import { isPrejoinPageVisible } from '../../prejoin/functions';
import { setAudioSettings } from '../../settings/actions.web';
import { getAvailableDevices } from '../devices/actions.web';
import { SET_AUDIO_MUTED } from '../media/actionTypes';
import { gumPending, setScreenshareMuted } from '../media/actions';
import {
    MEDIA_TYPE,
    VIDEO_TYPE
} from '../media/constants';
import { IGUMPendingState } from '../media/types';
import MiddlewareRegistry from '../redux/MiddlewareRegistry';

import {
    TRACK_ADDED,
    TRACK_MUTE_UNMUTE_FAILED,
    TRACK_NO_DATA_FROM_SOURCE,
    TRACK_REMOVED,
    TRACK_STOPPED,
    TRACK_UPDATED
} from './actionTypes';
import {
    createLocalTracksA,
    showNoDataFromSourceVideoError,
    toggleScreensharing,
    trackMuteUnmuteFailed,
    trackNoDataFromSourceNotificationInfoChanged
} from './actions.web';
import {
    getLocalJunctoAudioTrackSettings,
    getLocalTrack,
    getTrackByJunctoTrack, isUserInteractionRequiredForUnmute, logTracksForParticipant,
    setTrackMuted
} from './functions.web';
import { ITrack, ITrackOptions } from './types';

import './middleware.any';
import './subscriber.web';

/**
 * Middleware that captures LIB_DID_DISPOSE and LIB_DID_INIT actions and,
 * respectively, creates/destroys local media tracks. Also listens to
 * media-related actions and performs corresponding operations with tracks.
 *
 * @param {Store} store - The redux store.
 * @returns {Function}
 */
MiddlewareRegistry.register(store => next => action => {
    switch (action.type) {
    case TRACK_ADDED: {
        const { local } = action.track;

        // The devices list needs to be refreshed when no initial video permissions
        // were granted and a local video track is added by umuting the video.
        if (local) {
            store.dispatch(getAvailableDevices());
            break;
        }

        const result = next(action);
        const participantId = action.track?.participantId;

        if (participantId) {
            logTracksForParticipant(store.getState()['features/base/tracks'], participantId, 'Track added');
        }

        return result;
    }
    case TRACK_NO_DATA_FROM_SOURCE: {
        const result = next(action);

        _handleNoDataFromSourceErrors(store, action);

        return result;
    }

    case TRACK_REMOVED: {
        _removeNoDataFromSourceNotification(store, action.track);

        const result = next(action);
        const participantId = action.track?.junctoTrack?.getParticipantId();

        if (participantId && !action.track?.junctoTrack?.isLocal()) {
            logTracksForParticipant(store.getState()['features/base/tracks'], participantId, 'Track removed');
        }

        return result;
    }

    case TRACK_MUTE_UNMUTE_FAILED: {
        const { junctoTrack } = action.track;
        const muted = action.wasMuted;
        const isVideoTrack = junctoTrack.getType() !== MEDIA_TYPE.AUDIO;

        if (isVideoTrack && junctoTrack.getVideoType() === VIDEO_TYPE.DESKTOP) {
            store.dispatch(setScreenshareMuted(!muted));
        } else if (isVideoTrack) {
            APP.conference.setVideoMuteStatus();
        } else {
            APP.conference.updateAudioIconEnabled();
        }

        break;
    }

    case TRACK_STOPPED: {
        const { junctoTrack } = action.track;

        if (junctoTrack.getVideoType() === VIDEO_TYPE.DESKTOP) {
            store.dispatch(toggleScreensharing(false));
        }
        break;
    }

    case TRACK_UPDATED: {
        // TODO Remove the following calls to APP.UI once components interested
        // in track mute changes are moved into React and/or redux.

        const result = next(action);
        const state = store.getState();

        if (isPrejoinPageVisible(state)) {
            return result;
        }

        const { junctoTrack } = action.track;
        const participantID = junctoTrack.getParticipantId();
        const isVideoTrack = junctoTrack.type !== MEDIA_TYPE.AUDIO;
        const local = junctoTrack.isLocal();

        if (isVideoTrack) {
            if (local && !(junctoTrack.getVideoType() === VIDEO_TYPE.DESKTOP)) {
                APP.conference.setVideoMuteStatus();
            } else if (!local) {
                APP.UI.setVideoMuted(participantID);
            }
        } else if (local) {
            APP.conference.updateAudioIconEnabled();
        }

        if (typeof action.track?.muted !== 'undefined' && participantID && !local) {
            logTracksForParticipant(store.getState()['features/base/tracks'], participantID, 'Track updated');

            // Notify external API when remote participant mutes/unmutes themselves
            const mediaType = isVideoTrack
                ? (junctoTrack.getVideoType() === VIDEO_TYPE.DESKTOP ? 'desktop' : 'video')
                : 'audio';

            APP.API.notifyParticipantMuted(participantID, action.track.muted, mediaType, true);
        }

        return result;
    }
    case SET_AUDIO_MUTED: {
        if (!action.muted
                && isUserInteractionRequiredForUnmute(store.getState())) {
            return;
        }

        _setMuted(store, action);
        break;
    }
    }

    return next(action);
});

/**
 * Handles no data from source errors.
 *
 * @param {Store} store - The redux store in which the specified action is
 * dispatched.
 * @param {Action} action - The redux action dispatched in the specified store.
 * @private
 * @returns {void}
 */
function _handleNoDataFromSourceErrors(store: IStore, action: AnyAction) {
    const { getState, dispatch } = store;

    const track = getTrackByJunctoTrack(getState()['features/base/tracks'], action.track.junctoTrack);

    if (!track?.local) {
        return;
    }

    const { junctoTrack } = track;

    if (track.mediaType === MEDIA_TYPE.AUDIO && track.isReceivingData) {
        _removeNoDataFromSourceNotification(store, action.track);
    }

    if (track.mediaType === MEDIA_TYPE.VIDEO) {
        const { noDataFromSourceNotificationInfo = {} } = track;

        if (track.isReceivingData) {
            if (noDataFromSourceNotificationInfo.timeout) {
                clearTimeout(noDataFromSourceNotificationInfo.timeout);
                dispatch(trackNoDataFromSourceNotificationInfoChanged(junctoTrack, undefined));
            }

            // try to remove the notification if there is one.
            _removeNoDataFromSourceNotification(store, action.track);
        } else {
            if (noDataFromSourceNotificationInfo.timeout) {
                return;
            }

            const timeout = setTimeout(() => dispatch(showNoDataFromSourceVideoError(junctoTrack)), 5000);

            dispatch(trackNoDataFromSourceNotificationInfoChanged(junctoTrack, { timeout }));
        }
    }
}

/**
 * Removes the no data from source notification associated with the JunctoTrack if displayed.
 *
 * @param {Store} store - The redux store.
 * @param {Track} track - The redux action dispatched in the specified store.
 * @returns {void}
 */
function _removeNoDataFromSourceNotification({ getState, dispatch }: IStore, track: ITrack) {
    const t = getTrackByJunctoTrack(getState()['features/base/tracks'], track.junctoTrack);
    const { junctoTrack, noDataFromSourceNotificationInfo = {} } = t || {};

    if (noDataFromSourceNotificationInfo?.uid) {
        dispatch(hideNotification(noDataFromSourceNotificationInfo.uid));
        dispatch(trackNoDataFromSourceNotificationInfoChanged(junctoTrack, undefined));
    }
}

/**
 * Mutes or unmutes a local track with a specific media type.
 *
 * @param {Store} store - The redux store in which the specified action is
 * dispatched.
 * @param {Action} action - The redux action dispatched in the specified store.
 * @private
 * @returns {void}
 */
function _setMuted(store: IStore, { ensureTrack, muted }: {
    ensureTrack: boolean; muted: boolean; }) {
    const { dispatch, getState } = store;
    const state = getState();
    const localTrack = getLocalTrack(state['features/base/tracks'], MEDIA_TYPE.AUDIO, /* includePending */ true);

    if (localTrack) {
        // The `junctoTrack` property will have a value only for a localTrack for which `getUserMedia` has already
        // completed. If there's no `junctoTrack`, then the `muted` state will be applied once the `junctoTrack` is
        // created.
        const { junctoTrack } = localTrack;

        if (junctoTrack) {
            setTrackMuted(junctoTrack, muted, state, dispatch)
            .catch(() => {
                dispatch(trackMuteUnmuteFailed(localTrack, muted));
            });
        }
    } else if (!muted && ensureTrack) {
        // TODO(saghul): reconcile these 2 types.
        dispatch(gumPending([ MEDIA_TYPE.AUDIO ], IGUMPendingState.PENDING_UNMUTE));

        const createTrackOptions: ITrackOptions = {
            devices: [ MEDIA_TYPE.AUDIO ],
        };

        dispatch(createLocalTracksA(createTrackOptions)).then(() => {
            dispatch(gumPending([ MEDIA_TYPE.AUDIO ], IGUMPendingState.NONE));
            const updatedSettings = getLocalJunctoAudioTrackSettings(getState());

            dispatch(setAudioSettings(updatedSettings));
        });
    }
}
