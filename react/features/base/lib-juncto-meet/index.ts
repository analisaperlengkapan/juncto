// Re-export JunctoMeetJS from the library lib-juncto to (the other features
// of) the project juncto.
import JunctoMeetJS from './_';
export { JunctoMeetJS as default };

// XXX Re-export the properties exported by JunctoMeetJS in order to prevent
// undefined imported JunctoMeetJS. It may be caused by import cycles but I have
// not confirmed the theory.
export const analytics = JunctoMeetJS.analytics;
export const browser = JunctoMeetJS.util.browser;
export const JunctoConferenceErrors = JunctoMeetJS.errors.conference;
export const JunctoConferenceEvents = JunctoMeetJS.events.conference;
export const JunctoConnectionErrors = JunctoMeetJS.errors.connection;
export const JunctoConnectionEvents = JunctoMeetJS.events.connection;
export const JunctoConnectionQualityEvents = JunctoMeetJS.events.connectionQuality;
export const JunctoDetectionEvents = JunctoMeetJS.events.detection;
export const JunctoE2ePingEvents = JunctoMeetJS.events.e2eping;
export const JunctoMediaDevicesEvents = JunctoMeetJS.events.mediaDevices;
export const JunctoTrackStreamingStatus = JunctoMeetJS.constants.trackStreamingStatus;
export const JunctoRecordingConstants = JunctoMeetJS.constants.recording;
export const JunctoSIPVideoGWStatus = JunctoMeetJS.constants.sipVideoGW;
export const JunctoTrackErrors = JunctoMeetJS.errors.track;
export const JunctoTrackEvents = JunctoMeetJS.events.track;
export const RTCStatsEvents = JunctoMeetJS.events.rtcstats;
