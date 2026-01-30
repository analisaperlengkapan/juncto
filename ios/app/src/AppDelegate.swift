import UIKit
import Firebase
import JunctoMeetSDK

@main
class AppDelegate: UIResponder, UIApplicationDelegate {

    var window: UIWindow?

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        self.window = UIWindow(frame: UIScreen.main.bounds)

        let junctoMeet = JunctoMeet.sharedInstance()

        // junctoMeet.webRtcLoggingSeverity = .verbose

        junctoMeet.conferenceActivityType = "org.juncto.JunctoMeet.ios.conference" // Must match the one defined in Info.plist{}
        junctoMeet.customUrlScheme = "org.juncto.meet"
        junctoMeet.universalLinkDomains = ["meet.juncto.net", "alpha.juncto.net", "beta.meet.juncto.net"]

        junctoMeet.defaultConferenceOptions = JunctoMeetConferenceOptions.fromBuilder { builder in
            // For testing configOverrides a room needs to be set
            // builder.room = "https://meet.juncto.net/test0988test"

            builder.setFeatureFlag("welcomepage.enabled", withBoolean: true)
            builder.setFeatureFlag("ios.screensharing.enabled", withBoolean: true)
            builder.setFeatureFlag("ios.recording.enabled", withBoolean: true)
        }

        junctoMeet.application(application, didFinishLaunchingWithOptions: launchOptions ?? [:])

        if self.appContainsRealServiceInfoPlist() {
            print("Enabling Firebase")
            FirebaseApp.configure()
            Crashlytics.crashlytics().setCrashlyticsCollectionEnabled(!junctoMeet.isCrashReportingDisabled())
        }

        let vc = ViewController()
        self.window?.rootViewController = vc
        junctoMeet.showSplashScreen()

        self.window?.makeKeyAndVisible()

        return true
    }

    func applicationWillTerminate(_ application: UIApplication) {
        print("Application will terminate!")
        if let rootController = self.window?.rootViewController as? ViewController {
            rootController.terminate()
        }
    }

    // MARK: Linking delegate methods

    func application(_ application: UIApplication, continue userActivity: NSUserActivity, restorationHandler: @escaping ([UIUserActivityRestoring]?) -> Void) -> Bool {
        return JunctoMeet.sharedInstance().application(application, continue: userActivity, restorationHandler: restorationHandler)
    }

    func application(_ app: UIApplication, open url: URL, options: [UIApplication.OpenURLOptionsKey: Any] = [:]) -> Bool {
        if url.absoluteString.contains("google/link/?dismiss=1&is_weak_match=1") {
            return false
        }

        return JunctoMeet.sharedInstance().application(app, open: url, options: options)
    }

    func application(_ application: UIApplication, supportedInterfaceOrientationsFor window: UIWindow?) -> UIInterfaceOrientationMask {
        return JunctoMeet.sharedInstance().application(application, supportedInterfaceOrientationsFor: window)
    }
}

// Firebase utilities
extension AppDelegate {
    func appContainsRealServiceInfoPlist() -> Bool {
        return InfoPlistUtil.containsRealServiceInfoPlist(in: Bundle.main)
    }
}
