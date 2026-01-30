import { IReduxState } from '../../app/types';

/**
 * Checks if Juncto is running on Spot TV.
 *
 * @param {IReduxState} state - The redux state.
 * @returns {boolean} Whether or not Juncto is running on Spot TV.
 */
export function isSpotTV(state: IReduxState): boolean {
    const { defaultLocalDisplayName, iAmSpot } = state['features/base/config'] || {};

    return iAmSpot
        || navigator.userAgent.includes('JunctoSpot/') // Juncto Spot app
        || navigator.userAgent.includes('JunctoMeetingRooms/') // Juncto Meeting Rooms app
        || defaultLocalDisplayName === 'Meeting Room';
}
