package org.juncto.meet.sdk;

import android.content.Intent;
import android.os.Bundle;

/**
 * Wraps the name and extra data for events that were broadcasted locally.
 */
public class BroadcastAction {
    private static final String TAG = BroadcastAction.class.getSimpleName();

    private final Type type;
    private final Bundle data;

    public BroadcastAction(Intent intent) {
        this.type = Type.buildTypeFromAction(intent.getAction());
        this.data = intent.getExtras();
    }

    public Type getType() {
        return this.type;
    }

    public Bundle getData() {
        return this.data;
    }

    enum Type {
        SET_AUDIO_MUTED("org.juncto.meet.SET_AUDIO_MUTED"),
        HANG_UP("org.juncto.meet.HANG_UP"),
        SEND_ENDPOINT_TEXT_MESSAGE("org.juncto.meet.SEND_ENDPOINT_TEXT_MESSAGE"),
        TOGGLE_SCREEN_SHARE("org.juncto.meet.TOGGLE_SCREEN_SHARE"),
        RETRIEVE_PARTICIPANTS_INFO("org.juncto.meet.RETRIEVE_PARTICIPANTS_INFO"),
        OPEN_CHAT("org.juncto.meet.OPEN_CHAT"),
        CLOSE_CHAT("org.juncto.meet.CLOSE_CHAT"),
        SEND_CHAT_MESSAGE("org.juncto.meet.SEND_CHAT_MESSAGE"),
        SET_VIDEO_MUTED("org.juncto.meet.SET_VIDEO_MUTED"),
        SET_CLOSED_CAPTIONS_ENABLED("org.juncto.meet.SET_CLOSED_CAPTIONS_ENABLED"),
        TOGGLE_CAMERA("org.juncto.meet.TOGGLE_CAMERA"),
        SHOW_NOTIFICATION("org.juncto.meet.SHOW_NOTIFICATION"),
        HIDE_NOTIFICATION("org.juncto.meet.HIDE_NOTIFICATION"),
        START_RECORDING("org.juncto.meet.START_RECORDING"),
        STOP_RECORDING("org.juncto.meet.STOP_RECORDING"),
        OVERWRITE_CONFIG("org.juncto.meet.OVERWRITE_CONFIG"),
        SEND_CAMERA_FACING_MODE_MESSAGE("org.juncto.meet.SEND_CAMERA_FACING_MODE_MESSAGE");

        private final String action;

        Type(String action) {
            this.action = action;
        }

        public String getAction() {
            return action;
        }

        private static Type buildTypeFromAction(String action) {
            for (Type type : Type.values()) {
                if (type.action.equalsIgnoreCase(action)) {
                    return type;
                }
            }
            return null;
        }
    }
}
