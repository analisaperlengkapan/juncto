import ReducerRegistry from '../base/redux/ReducerRegistry';

import {
    SET_DETAILS
} from './actionTypes';
import { STATUSES } from './constants';

const DEFAULT_STATE = {
    disabledFeatures: [],
    status: STATUSES.ACTIVE
};

export interface IJunctoServiceState {
    [key: string]: any;
}

/**
 * Listen for actions that mutate the billing-counter state.
 */
ReducerRegistry.register<IJunctoServiceState>(
    'features/jaas', (state = DEFAULT_STATE, action): IJunctoServiceState => {
        switch (action.type) {

        case SET_DETAILS: {
            return action.payload;
        }

        default:
            return state;
        }
    }
);
