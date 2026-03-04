#!/bin/bash
sed -i 's/if \*room_id != my_loc {/if room_id != \&my_loc {/g' rust-app/backend/src/handlers/ws.rs
