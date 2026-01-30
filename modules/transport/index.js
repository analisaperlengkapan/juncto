// FIXME: change to '../API' when we update to webpack2. If we do this now all
// files from API modules will be included in external_api.js.
import { PostMessageTransportBackend, Transport } from '@juncto/js-utils/transport';

import { getJunctoMeetGlobalNS } from '../../react/features/base/util/helpers';
import { API_ID } from '../API/constants';


export {
    PostMessageTransportBackend,
    Transport
};

/**
 * Option for the default low level transport.
 *
 * @type {Object}
 */
const postisOptions = {};

if (typeof API_ID === 'number') {
    postisOptions.scope = `juncto_meet_external_api_${API_ID}`;
}

/**
 * The instance of Transport class that will be used by Juncto.
 *
 * @type {Transport}
 */
let transport;

/**
 * Returns the instance of Transport class that will be used by Juncto.
 *
 * @returns {Transport}
 */
export function getJunctoMeetTransport() {
    if (!transport) {
        transport = new Transport({ backend: new PostMessageTransportBackend({ postisOptions }) });
    }

    return transport;
}

/**
 * Sets the transport to passed transport.
 *
 * @param {Object} externalTransportBackend - The new transport.
 * @returns {void}
 */
getJunctoMeetGlobalNS().setExternalTransportBackend = externalTransportBackend =>
    transport.setBackend(externalTransportBackend);
