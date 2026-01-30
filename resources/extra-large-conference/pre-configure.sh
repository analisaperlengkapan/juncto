#!/bin/bash

SCRIPT_DIR=`dirname "$0"`
cd $SCRIPT_DIR

NUMBER_OF_INSTANCES=$1

if ! [[ $NUMBER_OF_INSTANCES =~ ^[0-9]+([.][0-9]+)?$ ]] ; then
   echo "error: Not a number param" >&2;
   exit 1
fi

echo "Will configure $NUMBER_OF_INSTANCES number of visitor prosodies"
set -e
set -x

JUNCOFO_HOSTNAME=$(echo get juncto-junctobridge/junctobridge-hostname | sudo debconf-communicate juncofo | cut -d' ' -f2-)

# Configure prosody instances
for (( i=1 ; i<=${NUMBER_OF_INSTANCES} ; i++ ));
do
    cp prosody-v.service.template /lib/systemd/system/prosody-v${i}.service
    sed -i "s/vX/v${i}/g" /lib/systemd/system/prosody-v${i}.service
    mkdir /etc/prosody-v${i}
    ln -s /etc/prosody/certs /etc/prosody-v${i}/certs
    cp prosody.cfg.lua.visitor.template /etc/prosody-v${i}/prosody.cfg.lua
    sed -i "s/vX/v${i}/g" /etc/prosody-v${i}/prosody.cfg.lua
    sed -i "s/junctomeet.example.com/$JUNCOFO_HOSTNAME/g" /etc/prosody-v${i}/prosody.cfg.lua
    # fix the ports
    sed -i "s/52691/5269${i}/g" /etc/prosody-v${i}/prosody.cfg.lua
    sed -i "s/52221/5222${i}/g" /etc/prosody-v${i}/prosody.cfg.lua
    sed -i "s/52801/5280${i}/g" /etc/prosody-v${i}/prosody.cfg.lua
    sed -i "s/52811/5281${i}/g" /etc/prosody-v${i}/prosody.cfg.lua
done

# Configure juncofo
HOCON_CONFIG="/etc/juncto/juncofo/juncofo.conf"
hocon -f $HOCON_CONFIG set "juncofo.bridge.selection-strategy" "VisitorSelectionStrategy"
hocon -f $HOCON_CONFIG set "juncofo.bridge.visitor-selection-strategy" "RegionBasedBridgeSelectionStrategy"
hocon -f $HOCON_CONFIG set "juncofo.bridge.participant-selection-strategy" "RegionBasedBridgeSelectionStrategy"
hocon -f $HOCON_CONFIG set "juncofo.bridge.topology-strategy" "VisitorTopologyStrategy"

PASS=$(hocon -f $HOCON_CONFIG get "juncofo.xmpp.client.password")
for (( i=1 ; i<=${NUMBER_OF_INSTANCES} ; i++ ));
do
  prosodyctl --config /etc/prosody-v${i}/prosody.cfg.lua register focus auth.meet.juncto $PASS
  hocon -f $HOCON_CONFIG set "juncofo.xmpp.visitors.v${i}.enabled" true
  hocon -f $HOCON_CONFIG set "juncofo.xmpp.visitors.v${i}.conference-service" "conference.v${i}.meet.juncto"
  hocon -f $HOCON_CONFIG set "juncofo.xmpp.visitors.v${i}.hostname" 127.0.0.1
  hocon -f $HOCON_CONFIG set "juncofo.xmpp.visitors.v${i}.port" 5222${i}
  hocon -f $HOCON_CONFIG set "juncofo.xmpp.visitors.v${i}.domain" "auth.meet.juncto"
  hocon -f $HOCON_CONFIG set "juncofo.xmpp.visitors.v${i}.xmpp-domain" "v${i}.meet.juncto"
  hocon -f $HOCON_CONFIG set "juncofo.xmpp.visitors.v${i}.password" "${PASS}"
  hocon -f $HOCON_CONFIG set "juncofo.xmpp.visitors.v${i}.disable-certificate-verification" true
done

for (( i=1 ; i<=${NUMBER_OF_INSTANCES} ; i++ ));
do
  service prosody-v${i} restart
done
service juncofo restart
