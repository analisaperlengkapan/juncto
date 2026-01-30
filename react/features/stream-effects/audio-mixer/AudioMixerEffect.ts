import JunctoMeetJS from '../../base/lib-juncto';
import { MEDIA_TYPE } from '../../base/media/constants';

/**
 * Class Implementing the effect interface expected by a JunctoLocalTrack.
 * The AudioMixerEffect, as the name implies, mixes two JunctoLocalTracks containing a audio track. First track is
 * provided at the moment of creation, second is provided through the effect interface.
 */
export class AudioMixerEffect {
    /**
     * JunctoLocalTrack that is going to be mixed into the track that uses this effect.
     */
    _mixAudio: any;

    /**
     * MediaStream resulted from mixing.
     */
    _mixedMediaStream: any;

    /**
     * MediaStreamTrack obtained from mixed stream.
     */
    _mixedMediaTrack: Object;

    /**
     * Original MediaStream from the JunctoLocalTrack that uses this effect.
     */
    _originalStream: Object;

    /**
     * MediaStreamTrack obtained from the original MediaStream.
     */
    _originalTrack: any;

    /**
     * Lib-juncto AudioMixer.
     */
    _audioMixer: any;

    /**
     * Creates AudioMixerEffect.
     *
     * @param {JunctoLocalTrack} mixAudio - JunctoLocalTrack which will be mixed with the original track.
     */
    constructor(mixAudio: any) {
        if (mixAudio.getType() !== MEDIA_TYPE.AUDIO) {
            throw new Error('AudioMixerEffect only supports audio JunctoLocalTracks; effect will not work!');
        }

        this._mixAudio = mixAudio;
    }

    /**
     * Checks if the JunctoLocalTrack supports this effect.
     *
     * @param {JunctoLocalTrack} sourceLocalTrack - Track to which the effect will be applied.
     * @returns {boolean} - Returns true if this effect can run on the specified track, false otherwise.
     */
    isEnabled(sourceLocalTrack: any) {
        // Both JunctoLocalTracks need to be audio i.e. contain an audio MediaStreamTrack
        return sourceLocalTrack.isAudioTrack() && this._mixAudio.isAudioTrack();
    }

    /**
     * Effect interface called by source JunctoLocalTrack, At this point a WebAudio ChannelMergerNode is created
     * and and the two associated MediaStreams are connected to it; the resulting mixed MediaStream is returned.
     *
     * @param {MediaStream} audioStream - Audio stream which will be mixed with _mixAudio.
     * @returns {MediaStream} - MediaStream containing both audio tracks mixed together.
     */
    // @ts-ignore
    startEffect(audioStream: MediaStream) {
        this._originalStream = audioStream;
        this._originalTrack = audioStream.getTracks()[0];

        this._audioMixer = JunctoMeetJS.createAudioMixer();
        this._audioMixer.addMediaStream(this._mixAudio.getOriginalStream());
        this._audioMixer.addMediaStream(this._originalStream);

        this._mixedMediaStream = this._audioMixer.start();
        this._mixedMediaTrack = this._mixedMediaStream.getTracks()[0];

        return this._mixedMediaStream;
    }

    /**
     * Reset the AudioMixer stopping it in the process.
     *
     * @returns {void}
     */
    stopEffect() {
        this._audioMixer.reset();
    }

    /**
     * Change the muted state of the effect.
     *
     * @param {boolean} muted - Should effect be muted or not.
     * @returns {void}
     */
    setMuted(muted: boolean) {
        this._originalTrack.enabled = !muted;
    }

    /**
     * Check whether or not this effect is muted.
     *
     * @returns {boolean}
     */
    isMuted() {
        return !this._originalTrack.enabled;
    }
}
