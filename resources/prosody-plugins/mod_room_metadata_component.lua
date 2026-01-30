-- This module implements a generic metadata storage system for rooms.
--
-- Component "metadata.junctomeet.example.com" "room_metadata_component"
--      muc_component = "conference.junctomeet.example.com"
--      breakout_rooms_component = "breakout.junctomeet.example.com"
local array = require 'util.array';
local filters = require 'util.filters';
local jid_node = require 'util.jid'.node;
local json = require 'util.json';
local st = require 'util.stanza';
local jid = require 'util.jid';

local util = module:require 'util';
local is_admin = util.is_admin;
local is_healthcheck_room = util.is_healthcheck_room;
local get_room_from_jid = util.get_room_from_jid;
local room_jid_match_rewrite = util.room_jid_match_rewrite;
local internal_room_jid_match_rewrite = util.internal_room_jid_match_rewrite;
local process_host_module = util.process_host_module;
local table_shallow_copy = util.table_shallow_copy;
local table_add = util.table_add;
local table_equals = util.table_equals;

local MUC_NS = 'http://jabber.org/protocol/muc';
local COMPONENT_IDENTITY_TYPE = 'room_metadata';
local FORM_KEY = 'muc#roominfo_junctometadata';

local muc_component_host = module:get_option_string('muc_component');

if muc_component_host == nil then
    module:log('error', 'No muc_component specified. No muc to operate on!');
    return;
end

local main_virtual_host = module:get_option_string('muc_mapper_domain_base');
if not main_virtual_host then
    module:log('warn', 'No muc_mapper_domain_base option set.');
    return;
end

local breakout_rooms_component_host = module:get_option_string('breakout_rooms_component');

module:log("info", "Starting room metadata for %s", muc_component_host);

local main_muc_module;

-- Utility functions

-- Returns json string with the metadata for the room.
-- @param room The room object.
-- @param metadata Optional metadata to use instead of the room's junctoMetadata.
function getMetadataJSON(room, metadata)
    local res, error = json.encode({
        type = COMPONENT_IDENTITY_TYPE,
        metadata = metadata or room.junctoMetadata or {}
    });

    if not res then
        module:log('error', 'Error encoding data room:%s', room.jid, error);
    end

    return res;
end

function broadcastMetadata(room, json_msg)
    if not json_msg then
        return;
    end

    for _, occupant in room:each_occupant() do
        send_metadata(occupant, room, json_msg)
    end
end

function send_metadata(occupant, room, json_msg)
    if not json_msg or is_admin(occupant.bare_jid) then
        local metadata_to_send = room.junctoMetadata or {};

        -- we want to send the main meeting participants only to juncofo
        if is_admin(occupant.bare_jid) then
            local participants;
            local moderators = array();

            if room._data.participants then
                participants = array();
                participants:append(room._data.participants);
            end

            if room._data.moderator_id then
                moderators:push(room._data.moderator_id);
            end

            if room._data.moderators then
                moderators:append(room._data.moderators);
            end

            metadata_to_send = table_shallow_copy(metadata_to_send);
            metadata_to_send.participants = participants;
            metadata_to_send.moderators = moderators;

            module:log('info', 'Sending metadata to juncofo room=%s,meeting_id=%s', room.jid, room._data.meetingId);
        end

        json_msg = getMetadataJSON(room, metadata_to_send);
    end

    local stanza = st.message({ from = module.host; to = occupant.jid; })
         :tag('json-message', {
             xmlns = 'http://juncto.org/junctomeet',
             room = internal_room_jid_match_rewrite(room.jid)
         }):text(json_msg):up();
    module:send(stanza);
end

-- Handling events

function room_created(event)
    local room = event.room;

    if is_healthcheck_room(room.jid) then
        return;
    end

    if not room.junctoMetadata then
        room.junctoMetadata = {};
    end

    room.sent_initial_metadata = {};
end

function on_message(event)
    local session = event.origin;

    -- Check the type of the incoming stanza to avoid loops:
    if event.stanza.attr.type == 'error' then
        return; -- We do not want to reply to these, so leave.
    end

    if not session or not session.juncto_web_query_room then
        return false;
    end

    local message = event.stanza:get_child(COMPONENT_IDENTITY_TYPE, 'http://juncto.org/junctomeet');
    local messageText = message:get_text();

    if not message or not messageText then
        return false;
    end

    local roomJid = message.attr.room;
    local room = get_room_from_jid(room_jid_match_rewrite(roomJid));

    if not room then
        module:log('warn', 'No room found for %s/%s',
                session.juncto_web_query_prefix, session.juncto_web_query_room);
        return false;
    end

    -- check that the participant requesting is a moderator and is an occupant in the room
    local from = event.stanza.attr.from;
    local occupant = room:get_occupant_by_real_jid(from);

    if not occupant then
        module:log('warn', 'No occupant %s found for %s', from, room.jid);
        return false;
    end

    local jsonData, error = json.decode(messageText);
    if jsonData == nil then -- invalid JSON
        module:log("error", "Invalid JSON message: %s error:%s", messageText, error);
        return false;
    end

    if jsonData.key == nil or jsonData.data == nil then
        module:log("error", "Invalid JSON payload, key or data are missing: %s", messageText);
        return false;
    end

    if occupant.role ~= 'moderator' then
        -- will return a non nil filtered data to use, if it is nil, it is not allowed
        local res = module:context(main_virtual_host):fire_event('juncto-metadata-allow-moderation',
                { room = room; actor = occupant; key = jsonData.key ; data = jsonData.data; session = session; });

        if not res then
            module:log('warn', 'Occupant %s is not moderator and not allowed this operation for %s', from, room.jid);
            return false;
        end

        jsonData.data = res;
    end

    local old_value = room.junctoMetadata[jsonData.key];
    if not table_equals(old_value, jsonData.data) then
        room.junctoMetadata[jsonData.key] = jsonData.data;

        module:log('info', 'Metadata key "%s" updated by %s in room:%s,meeting_id:%s', jsonData.key, from, room.jid, room._data.meetingId);
        broadcastMetadata(room, getMetadataJSON(room));

        -- fire and event for the change
        main_muc_module:fire_event('juncto-metadata-updated', { room = room; actor = occupant; key = jsonData.key; });
    end

    return true;
end

-- Module operations

-- handle messages to this component
module:hook("message/host", on_message);

-- operates on already loaded main muc module
function process_main_muc_loaded(main_muc, host_module)
    main_muc_module = host_module;

    module:log('debug', 'Main muc loaded');
    module:log("info", "Hook to muc events on %s", muc_component_host);

    host_module:hook("muc-room-created", room_created, -1);

    -- The room metadata was updated internally (from another module).
    host_module:hook("room-metadata-changed", function(event)
        local room = event.room;
        local json_msg = getMetadataJSON(room);

        module:log('info', 'Metadata changed internally in room:%s,meeting_id:%s - broadcasting data:%s', room.jid, room._data.meetingId, json_msg);
        broadcastMetadata(room, json_msg);
    end);

    -- TODO: Once clients update to read/write metadata for startMuted policy we can drop this
    -- this is to convert presence settings from old clients to metadata
    host_module:hook('muc-broadcast-presence', function (event)
        local actor, occupant, room, stanza, x = event.actor, event.occupant, event.room, event.stanza, event.x;

        if is_healthcheck_room(room.jid) or occupant.role ~= 'moderator' then
            return;
        end

        local startMuted = stanza:get_child('startmuted', 'http://juncto.org/junctomeet/start-muted');

        if not startMuted then
            return;
        end

        if not room.junctoMetadata then
            room.junctoMetadata = {};
        end

        local startMutedMetadata = room.junctoMetadata.startMuted or {};

        local audioNewValue = startMuted.attr.audio == 'true';
        local videoNewValue = startMuted.attr.video == 'true';
        local send_update = false;

        if startMutedMetadata.audio ~= audioNewValue then
            startMutedMetadata.audio = audioNewValue;
            send_update = true;
        end
        if startMutedMetadata.video ~= videoNewValue then
            startMutedMetadata.video = videoNewValue;
            send_update = true;
        end

        if send_update then
            room.junctoMetadata.startMuted = startMutedMetadata;

            host_module:fire_event('room-metadata-changed', { room = room; });
        end
    end);

    -- The the connection jid for authenticated users (like juncofo) stays the same,
    -- so leaving and re-joining will result not sending metatadata again.
    -- Make sure we clear the sent_initial_metadata entry for the occupant on leave.
    host_module:hook("muc-occupant-left", function(event)
        local room, occupant = event.room, event.occupant;

        if room.sent_initial_metadata then
            room.sent_initial_metadata[jid.bare(event.occupant.jid)] = nil;
        end
    end);
end

-- process or waits to process the main muc component
process_host_module(muc_component_host, function(host_module, host)
    local muc_module = prosody.hosts[host].modules.muc;

    if muc_module then
        process_main_muc_loaded(muc_module, host_module);
    else
        module:log('debug', 'Will wait for muc to be available');
        prosody.hosts[host].events.add_handler('module-loaded', function(event)
            if (event.module == 'muc') then
                process_main_muc_loaded(prosody.hosts[host].modules.muc, host_module);
            end
        end);
    end
end);

-- breakout rooms support
function process_breakout_muc_loaded(breakout_muc, host_module)
    module:log('debug', 'Breakout rooms muc loaded');
    module:log("info", "Hook to muc events on %s", breakout_rooms_component_host);

    host_module:hook("muc-room-created", room_created, -1);
end

if breakout_rooms_component_host then
    process_host_module(breakout_rooms_component_host, function(host_module, host)
        local muc_module = prosody.hosts[host].modules.muc;

        if muc_module then
            process_breakout_muc_loaded(muc_module, host_module);
        else
            module:log('debug', 'Will wait for muc to be available');
            prosody.hosts[host].events.add_handler('module-loaded', function(event)
                if (event.module == 'muc') then
                    process_breakout_muc_loaded(prosody.hosts[host].modules.muc, host_module);
                end
            end);
        end
    end);
end

-- Send a message update for metadata before sending the first self presence
function filter_stanza(stanza, session)
    if not stanza.attr or not stanza.attr.to or stanza.name ~= 'presence' or stanza.attr.type == 'unavailable' then
        return stanza;
    end

    local bare_to = jid.bare(stanza.attr.to);
    local muc_x = stanza:get_child('x', MUC_NS..'#user');
    if not muc_x or not presence_check_status(muc_x, '110') then
        return stanza;
    end

    local room = get_room_from_jid(room_jid_match_rewrite(jid.bare(stanza.attr.from)));

    if not room or not room.sent_initial_metadata or is_healthcheck_room(room.jid) then
        return stanza;
    end

    if room.sent_initial_metadata[bare_to] then
        return stanza;
    end

    local occupant;
    for _, o in room:each_occupant() do
        if o.bare_jid == bare_to then
            occupant = o;
        end
    end

    if not occupant then
        module:log('warn', 'No occupant %s found for %s', bare_to, room.jid);
        return stanza;
    end

    room.sent_initial_metadata[bare_to] = true;

    send_metadata(occupant, room);

    return stanza;
end
function filter_session(session)
    -- domain mapper is filtering on default priority 0
    -- allowners is -1 and we need it after that, permissions is -2
    filters.add_filter(session, 'stanzas/out', filter_stanza, -3);
end

-- enable filtering presences
filters.add_filter_hook(filter_session);

process_host_module(main_virtual_host, function(host_module)
    module:context(host_module.host):fire_event('juncto-add-identity', {
        name = 'room_metadata'; host = module.host;
    });
end);
