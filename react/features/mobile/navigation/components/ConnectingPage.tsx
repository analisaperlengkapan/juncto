import React from 'react';
import { useTranslation } from 'react-i18next';
import { Text, View, ViewStyle } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';

import JunctoScreen from '../../../base/modal/components/JunctoScreen';
import LoadingIndicator from '../../../base/react/components/native/LoadingIndicator';

import { TEXT_COLOR, navigationStyles } from './styles';


const ConnectingPage = () => {
    const { t } = useTranslation();

    return (
        <JunctoScreen style = { navigationStyles.connectingScreenContainer }>
            <View style = { navigationStyles.connectingScreenContent as ViewStyle }>
                <SafeAreaView>
                    <LoadingIndicator
                        color = { TEXT_COLOR }
                        size = 'large' />
                    <Text style = { navigationStyles.connectingScreenText }>
                        { t('connectingOverlay.joiningRoom') }
                    </Text>
                </SafeAreaView>
            </View>
        </JunctoScreen>
    );
};

export default ConnectingPage;
