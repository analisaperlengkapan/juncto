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

#import "LogUtils.h"
#import "JunctoMeetLogger.h"
#import "JunctoMeetBaseLogHandler+Private.h"


@implementation JunctoMeetLogger

/**
* This gets called automagically when the program starts.
*/
__attribute__((constructor))
static void initializeLogger() {
    NSString *mainBundleId = [NSBundle mainBundle].bundleIdentifier;
    DDOSLogger *osLogger = [[DDOSLogger alloc] initWithSubsystem:mainBundleId category:@"JunctoMeetSDK"];
    [DDLog addLogger:osLogger];
}

+ (void)addHandler:(JunctoMeetBaseLogHandler *)handler {
    [DDLog addLogger:handler.logger];
}

+ (void)removeHandler:(JunctoMeetBaseLogHandler *)handler {
    [DDLog removeLogger:handler.logger];
}

@end
