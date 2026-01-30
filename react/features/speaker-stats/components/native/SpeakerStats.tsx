import React, { useEffect } from 'react';
import { useDispatch } from 'react-redux';

import JunctoScreen from '../../../base/modal/components/JunctoScreen';
import { resetSearchCriteria } from '../../actions.native';

import SpeakerStatsList from './SpeakerStatsList';
import SpeakerStatsSearch from './SpeakerStatsSearch';
import style from './styles';

/**
 * Component that renders the list of speaker stats.
 *
 * @returns {React$Element<any>}
 */
const SpeakerStats = () => {
    const dispatch = useDispatch();

    useEffect(() => {
        dispatch(resetSearchCriteria());
    }, []);

    return (
        <JunctoScreen
            style = { style.speakerStatsContainer }>
            <SpeakerStatsSearch />
            <SpeakerStatsList />
        </JunctoScreen>
    );
};

export default SpeakerStats;
