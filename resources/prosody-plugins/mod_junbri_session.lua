local json = require 'cjson';

local util = module:require 'util';
local room_jid_match_rewrite = util.room_jid_match_rewrite;
local get_room_from_jid = util.get_room_from_jid;

-- This needs to be attached to the main virtual host and the virtual host where juncofo is connected and authenticated.
-- The first pass is the iq coming from the client where we get the creator and attach it to the app_data.
-- The second pass is juncofo approving that and inviting junbri where we attach the session_id information to app_data
local function attachJunbriSessionId(event)
local stanza = event.stanza;
    if stanza.name == "iq" then
        local junbri = stanza:get_child('junbri', 'http://juncto.org/protocol/junbri');
        if junbri then
            if junbri.attr.action == 'start' then

                local update_app_data = false;
                local app_data = junbri.attr.app_data;
                if app_data then
                    app_data = json.decode(app_data);
                else
                    app_data = {};
                end
                if app_data.file_recording_metadata == nil then
                    app_data.file_recording_metadata = {};
                end

                if junbri.attr.room then
                    local junbri_room = junbri.attr.room;
                    junbri_room = room_jid_match_rewrite(junbri_room)
                    local room = get_room_from_jid(junbri_room);
                    if room then
                        local conference_details = {};
                        conference_details["session_id"] = room._data.meetingId;
                        app_data.file_recording_metadata.conference_details = conference_details;
                        update_app_data = true;
                    end
                else
                    -- no room is because the iq received by the initiator in the room
                    local session = event.origin;
                    -- if a token is provided, add data to app_data
                    if session ~= nil then
                        local initiator = {};

                        if session.juncto_meet_context_user ~= nil then
                            initiator.id = session.juncto_meet_context_user.id;
                        else
                            initiator.id = session.granted_juncto_meet_context_user_id;
                        end

                        initiator.group
                            = session.juncto_meet_context_group or session.granted_juncto_meet_context_group_id;

                        app_data.file_recording_metadata.initiator = initiator
                        update_app_data = true;
                    end

                end

                if update_app_data then
                    app_data = json.encode(app_data);
                    junbri.attr.app_data = app_data;
                    junbri:up()
                    stanza:up()
                end
            end
        end
    end
end

module:hook('pre-iq/full', attachJunbriSessionId);
