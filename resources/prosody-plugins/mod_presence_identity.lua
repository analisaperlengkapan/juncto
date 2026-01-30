local stanza = require "util.stanza";
local update_presence_identity = module:require "util".update_presence_identity;

-- For all received presence messages, if the juncto_meet_context_(user|group)
-- values are set in the session, then insert them into the presence messages
-- for that session.
function on_message(event)
    local stanza, session = event.stanza, event.origin;
    if stanza and session then
          update_presence_identity(
              stanza,
              session.juncto_meet_context_user,
              session.juncto_meet_context_group
          );
    end
end

module:hook("pre-presence/bare", on_message);
module:hook("pre-presence/full", on_message);
