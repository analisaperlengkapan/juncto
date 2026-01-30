import { getBundleId } from 'react-native-device-info';

/**
 * BUndle ids for the Juncto apps.
 */
const JUNCTO_MEET_APPS = [

    // iOS app.
    'com.atlassian.JunctoMeet.ios',

    // Android + iOS (testing) app.
    'org.juncto.meet',

    // Android debug app.
    'org.juncto.meet.debug'
];

/**
 * Checks whether we are loaded in iframe. In the mobile case we treat SDK
 * consumers as the web treats iframes.
 *
 * @returns {boolean} Whether the current app is a Juncto app.
 */
export function isEmbedded(): boolean {
    return !JUNCTO_MEET_APPS.includes(getBundleId());
}
