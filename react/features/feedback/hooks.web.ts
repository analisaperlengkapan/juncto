import { useSelector } from 'react-redux';

import FeedbackButtonWeb from './components/FeedbackButton.web';
import { shouldSendJunctoServiceFeedbackMetadata } from './functions.web';

const feedback = {
    key: 'feedback',
    Content: FeedbackButtonWeb,
    group: 4
};

/**
 * A hook that returns the feedback button if it is enabled and undefined otherwise.
 *
 *  @returns {Object | undefined}
 */
export function useFeedbackButton() {
    const visible = useSelector(shouldSendJunctoServiceFeedbackMetadata);

    if (visible) {
        return feedback;
    }
}
