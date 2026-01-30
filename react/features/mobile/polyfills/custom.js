import { NativeModules } from 'react-native';


global.JUNCTO_MEET_LITE_SDK = Boolean(NativeModules.AppInfo.isLiteSDK);
