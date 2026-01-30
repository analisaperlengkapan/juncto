package org.juncto.meet;

import android.net.Uri;
import android.util.Log;

import com.google.firebase.crashlytics.FirebaseCrashlytics;

import org.juncto.meet.sdk.JunctoMeet;
import org.juncto.meet.sdk.JunctoMeetActivity;

/**
 * Helper class to initialize Google related services and functionality.
 * This functionality is compiled conditionally and called via reflection, that's why it was
 * extracted here.
 *
 * "Libre builds" (builds with the LIBRE_BUILD flag set) will not include this file.
 */
final class GoogleServicesHelper {
    public static void initialize(JunctoMeetActivity activity) {
        if (BuildConfig.GOOGLE_SERVICES_ENABLED) {
            Log.d(activity.getClass().getSimpleName(), "Initializing Google Services");

            FirebaseCrashlytics.getInstance().setCrashlyticsCollectionEnabled(!JunctoMeet.isCrashReportingDisabled(activity));
        }
    }
}
