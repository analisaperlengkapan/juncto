/*
 * Copyright @ 2019-present Juncto, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package org.juncto.meet.sdk;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.content.res.Configuration;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.view.View;
import android.view.ViewGroup;
import android.view.Window;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.view.ViewCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.localbroadcastmanager.content.LocalBroadcastManager;

import com.facebook.react.modules.core.PermissionListener;

import org.juncto.meet.sdk.log.JunctoMeetLogger;

import java.util.HashMap;

/**
 * A base activity for SDK users to embed.  It contains all the required wiring
 * between the {@code JunctoMeetView} and the Activity lifecycle methods.
 *
 * In this activity we use a single {@code JunctoMeetView} instance. This
 * instance gives us access to a view which displays the welcome page and the
 * conference itself. All lifecycle methods associated with this Activity are
 * hooked to the React Native subsystem via proxy calls through the
 * {@code JunctoMeetActivityDelegate} static methods.
 */
public class JunctoMeetActivity extends AppCompatActivity
    implements JunctoMeetActivityInterface {

    protected static final String TAG = JunctoMeetActivity.class.getSimpleName();

    private static final String ACTION_JUNCTO_MEET_CONFERENCE = "org.juncto.meet.CONFERENCE";
    private static final String JUNCTO_MEET_CONFERENCE_OPTIONS = "JunctoMeetConferenceOptions";

    private boolean isReadyToClose;

    private final BroadcastReceiver broadcastReceiver = new BroadcastReceiver() {
        @Override
        public void onReceive(Context context, Intent intent) {
            onBroadcastReceived(intent);
        }
    };

    /**
     * Instance of the {@link JunctoMeetView} which this activity will display.
     */
    private JunctoMeetView junctoView;

    // Helpers for starting the activity
    //

    public static void launch(Context context, JunctoMeetConferenceOptions options) {
        Intent intent = new Intent(context, JunctoMeetActivity.class);
        intent.setAction(ACTION_JUNCTO_MEET_CONFERENCE);
        intent.putExtra(JUNCTO_MEET_CONFERENCE_OPTIONS, options);
        if (!(context instanceof Activity)) {
            intent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        }
        context.startActivity(intent);
    }

    public static void launch(Context context, String url) {
        JunctoMeetConferenceOptions options
            = new JunctoMeetConferenceOptions.Builder().setRoom(url).build();
        launch(context, options);
    }

    public static void addTopBottomInsets(@NonNull Window w, @NonNull View v) {

        View decorView = w.getDecorView();

        decorView.post(() -> {
            WindowInsetsCompat insets = ViewCompat.getRootWindowInsets(decorView);
            if (insets != null) {
                ViewGroup.MarginLayoutParams params = (ViewGroup.MarginLayoutParams) v.getLayoutParams();
                params.topMargin = insets.getInsets(WindowInsetsCompat.Type.systemBars()).top;
                params.bottomMargin = insets.getInsets(WindowInsetsCompat.Type.systemBars()).bottom;
                v.setLayoutParams(params);

                decorView.setOnApplyWindowInsetsListener((view, windowInsets) -> {
                    view.setBackgroundColor(JunctoMeetView.BACKGROUND_COLOR);

                    return windowInsets;
                });
            }
        });
    }

    // Overrides
    //

    @Override
    public void onConfigurationChanged(Configuration newConfig) {
        super.onConfigurationChanged(newConfig);
        Intent intent = new Intent("onConfigurationChanged");
        intent.putExtra("newConfig", newConfig);
        this.sendBroadcast(intent);
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        // ReactInstanceManager is now initialized by JunctoInitializer during application startup
        // Just call onHostResume since the manager is already ready
        JunctoMeetActivityDelegate.onHostResume(this);

        setContentView(R.layout.activity_juncto_meet);

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.VANILLA_ICE_CREAM
            && getApplicationInfo().targetSdkVersion >= Build.VERSION_CODES.VANILLA_ICE_CREAM) {
            addTopBottomInsets(getWindow(), findViewById(android.R.id.content));
        }

        this.junctoView = findViewById(R.id.junctoView);

        registerForBroadcastMessages();

        if (!extraInitialize()) {
            initialize();
        }
    }

    @Override
    public void onResume() {
        super.onResume();
        JunctoMeetActivityDelegate.onHostResume(this);
    }

    @Override
    public void onStop() {
        JunctoMeetActivityDelegate.onHostPause(this);
        super.onStop();
    }

    @Override
    public void onDestroy() {
        JunctoMeetLogger.i("onDestroy()");

        // Here we are trying to handle the following corner case: an application using the SDK
        // is using this Activity for displaying meetings, but there is another "main" Activity
        // with other content. If this Activity is "swiped out" from the recent list we will get
        // Activity#onDestroy() called without warning. At this point we can try to leave the
        // current meeting, but when our view is detached from React the JS <-> Native bridge won't
        // be operational so the external API won't be able to notify the native side that the
        // conference terminated. Thus, try our best to clean up.
        if (!isReadyToClose) {
            JunctoMeetLogger.i("onDestroy(): leaving...");
            leave();
        }

        this.junctoView = null;

        if (AudioModeModule.useConnectionService()) {
            ConnectionService.abortConnections();
        }
        JunctoMeetOngoingConferenceService.abort(this);

        LocalBroadcastManager.getInstance(this).unregisterReceiver(broadcastReceiver);

        JunctoMeetActivityDelegate.onHostDestroy(this);

        super.onDestroy();
    }

    @Override
    public void finish() {
        if (!isReadyToClose) {
            JunctoMeetLogger.i("finish(): leaving...");
            leave();
        }

        JunctoMeetLogger.i("finish(): finishing...");
        super.finish();
    }

    // Helper methods
    //

    protected JunctoMeetView getJunctoView() {
        return junctoView;
    }

    public void join(@Nullable String url) {
        JunctoMeetConferenceOptions options
            = new JunctoMeetConferenceOptions.Builder()
            .setRoom(url)
            .build();
        join(options);
    }

    public void join(JunctoMeetConferenceOptions options) {
        if (this.junctoView != null) {
            this.junctoView.join(options);
        } else {
            JunctoMeetLogger.w("Cannot join, view is null");
        }
    }

    protected void leave() {
        if (this.junctoView != null) {
            this.junctoView.abort();
        } else {
            JunctoMeetLogger.w("Cannot leave, view is null");
        }
    }

    private @Nullable
    JunctoMeetConferenceOptions getConferenceOptions(Intent intent) {
        String action = intent.getAction();

        if (Intent.ACTION_VIEW.equals(action)) {
            Uri uri = intent.getData();
            if (uri != null) {
                return new JunctoMeetConferenceOptions.Builder().setRoom(uri.toString()).build();
            }
        } else if (ACTION_JUNCTO_MEET_CONFERENCE.equals(action)) {
            return intent.getParcelableExtra(JUNCTO_MEET_CONFERENCE_OPTIONS);
        }

        return null;
    }

    /**
     * Helper function called during activity initialization. If {@code true} is returned, the
     * initialization is delayed and the {@link JunctoMeetActivity#initialize()} method is not
     * called. In this case, it's up to the subclass to call the initialize method when ready.
     * <p>
     * This is mainly required so we do some extra initialization in the Juncto app.
     *
     * @return {@code true} if the initialization will be delayed, {@code false} otherwise.
     */
    protected boolean extraInitialize() {
        return false;
    }

    protected void initialize() {
        // Join the room specified by the URL the app was launched with.
        // Joining without the room option displays the welcome page.
        join(getConferenceOptions(getIntent()));
    }

    protected void onConferenceJoined(HashMap<String, Object> extraData) {
        JunctoMeetLogger.i("Conference joined: " + extraData);
        // Launch the service for the ongoing notification.
        JunctoMeetOngoingConferenceService.launch(this, extraData);
    }

    protected void onConferenceTerminated(HashMap<String, Object> extraData) {
        JunctoMeetLogger.i("Conference terminated: " + extraData);
    }

    protected void onConferenceWillJoin(HashMap<String, Object> extraData) {
        JunctoMeetLogger.i("Conference will join: " + extraData);
    }

    protected void onParticipantJoined(HashMap<String, Object> extraData) {
        try {
            JunctoMeetLogger.i("Participant joined: ", extraData);
        } catch (Exception e) {
            JunctoMeetLogger.w("Invalid participant joined extraData", e);
        }
    }

    protected void onParticipantLeft(HashMap<String, Object> extraData) {
        try {
            JunctoMeetLogger.i("Participant left: ", extraData);
        } catch (Exception e) {
            JunctoMeetLogger.w("Invalid participant left extraData", e);
        }
    }

    protected void onReadyToClose() {
        JunctoMeetLogger.i("SDK is ready to close");
        isReadyToClose = true;
        finish();
    }

//    protected void onTranscriptionChunkReceived(HashMap<String, Object> extraData) {
//        JunctoMeetLogger.i("Transcription chunk received: " + extraData);
//    }

//    protected void onCustomButtonPressed(HashMap<String, Object> extraData) {
//         JunctoMeetLogger.i("Custom button pressed: " + extraData);
//     }

//     protected void onConferenceUniqueIdSet(HashMap<String, Object> extraData) {
//         JunctoMeetLogger.i("Conference unique id set: " + extraData);
//     }

//     protected void onRecordingStatusChanged(HashMap<String, Object> extraData) {
//       JunctoMeetLogger.i("Recording status changed: " + extraData);
//     }

    // Activity lifecycle methods
    //

    @Override
    public void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);

        JunctoMeetActivityDelegate.onActivityResult(this, requestCode, resultCode, data);
    }

    @Override
    public void onBackPressed() {
        JunctoMeetActivityDelegate.onBackPressed();
    }

    @Override
    public void onNewIntent(Intent intent) {
        super.onNewIntent(intent);

        JunctoMeetConferenceOptions options;

        if ((options = getConferenceOptions(intent)) != null) {
            join(options);
            return;
        }

        JunctoMeetActivityDelegate.onNewIntent(intent);
    }

    @Override
    protected void onUserLeaveHint() {
        if (this.junctoView != null) {
            this.junctoView.enterPictureInPicture();
        }
    }

    // JunctoMeetActivityInterface
    //

    @Override
    public void requestPermissions(String[] permissions, int requestCode, PermissionListener listener) {
        JunctoMeetActivityDelegate.requestPermissions(this, permissions, requestCode, listener);
    }

    @SuppressLint("MissingSuperCall")
    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        JunctoMeetActivityDelegate.onRequestPermissionsResult(requestCode, permissions, grantResults);
    }

    private void registerForBroadcastMessages() {
        IntentFilter intentFilter = new IntentFilter();

        for (BroadcastEvent.Type type : BroadcastEvent.Type.values()) {
            intentFilter.addAction(type.getAction());
        }

        LocalBroadcastManager.getInstance(this).registerReceiver(broadcastReceiver, intentFilter);
    }

    private void onBroadcastReceived(Intent intent) {
        if (intent != null) {
            BroadcastEvent event = new BroadcastEvent(intent);

            switch (event.getType()) {
                case CONFERENCE_JOINED:
                    onConferenceJoined(event.getData());
                    break;
                case CONFERENCE_WILL_JOIN:
                    onConferenceWillJoin(event.getData());
                    break;
                case CONFERENCE_TERMINATED:
                    onConferenceTerminated(event.getData());
                    break;
                case PARTICIPANT_JOINED:
                    onParticipantJoined(event.getData());
                    break;
                case PARTICIPANT_LEFT:
                    onParticipantLeft(event.getData());
                    break;
                case READY_TO_CLOSE:
                    onReadyToClose();
                    break;
                // case TRANSCRIPTION_CHUNK_RECEIVED:
                //    onTranscriptionChunkReceived(event.getData());
                //    break;
                // case CUSTOM_BUTTON_PRESSED:
                //    onCustomButtonPressed(event.getData());
                //    break;
                // case CONFERENCE_UNIQUE_ID_SET:
                //     onConferenceUniqueIdSet(event.getData());
                //     break;
                // case RECORDING_STATUS_CHANGED:
                //     onRecordingStatusChanged(event.getData());
                //     break;
            }
        }
    }
}
