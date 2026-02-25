use leptos::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    MediaStream, RtcIceCandidate, RtcIceCandidateInit, RtcPeerConnection, RtcPeerConnectionIceEvent,
    RtcSdpType, RtcSessionDescriptionInit, RtcTrackEvent,
};

type PeerId = String;

#[derive(Clone)]
pub struct WebRTCManager {
    peers: Rc<RefCell<HashMap<PeerId, RtcPeerConnection>>>,
    send_signal: Rc<dyn Fn(shared::ClientMessage)>,
    on_track: Rc<dyn Fn(PeerId, MediaStream)>,
    local_stream: Signal<Option<MediaStream>>,
    local_screen_stream: Signal<Option<MediaStream>>,
    my_id: Signal<Option<String>>,
}

impl WebRTCManager {
    pub fn new(
        send_signal: impl Fn(shared::ClientMessage) + 'static,
        on_track: impl Fn(PeerId, MediaStream) + 'static,
        local_stream: Signal<Option<MediaStream>>,
        local_screen_stream: Signal<Option<MediaStream>>,
        my_id: Signal<Option<String>>,
    ) -> Self {
        Self {
            peers: Rc::new(RefCell::new(HashMap::new())),
            send_signal: Rc::new(send_signal),
            on_track: Rc::new(on_track),
            local_stream,
            local_screen_stream,
            my_id,
        }
    }

    fn create_peer_connection(&self, peer_id: &str) -> Result<RtcPeerConnection, JsValue> {
        let config = web_sys::RtcConfiguration::new();
        let ice_servers = js_sys::Array::new();
        // Use Google STUN for now
        let stun = web_sys::RtcIceServer::new();
        let urls = js_sys::Array::new();
        urls.push(&JsValue::from_str("stun:stun.l.google.com:19302"));
        stun.set_urls(&urls);
        ice_servers.push(&stun);
        config.set_ice_servers(&ice_servers);

        let pc = RtcPeerConnection::new_with_configuration(&config)?;

        // Add local tracks (camera)
        if let Some(stream) = self.local_stream.get_untracked() {
            let tracks = stream.get_tracks();
            for track in tracks.iter() {
                let track = track.unchecked_ref::<web_sys::MediaStreamTrack>();
                let streams = js_sys::Array::new();
                streams.push(&stream);
                let _ = pc.add_track(track, &stream, &streams);
            }
        }

        // Add local screen tracks (screen share)
        if let Some(stream) = self.local_screen_stream.get_untracked() {
            let tracks = stream.get_tracks();
            for track in tracks.iter() {
                let track = track.unchecked_ref::<web_sys::MediaStreamTrack>();
                let streams = js_sys::Array::new();
                streams.push(&stream);
                let _ = pc.add_track(track, &stream, &streams);
            }
        }

        // On Ice Candidate
        let send_signal = self.send_signal.clone();
        let peer_id_clone = peer_id.to_string();
        let on_ice_candidate = Closure::wrap(Box::new(move |ev: RtcPeerConnectionIceEvent| {
            if let Some(candidate) = ev.candidate() {
                let msg = shared::ClientMessage::IceCandidate {
                    target_id: peer_id_clone.clone(),
                    candidate: candidate.candidate(),
                    sdp_mid: candidate.sdp_mid(),
                    sdp_m_line_index: candidate.sdp_m_line_index(),
                };
                (send_signal)(msg);
            }
        }) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>);
        pc.set_onicecandidate(Some(on_ice_candidate.as_ref().unchecked_ref()));
        on_ice_candidate.forget();

        // On Track
        let on_track_cb = self.on_track.clone();
        let peer_id_clone_2 = peer_id.to_string();
        let on_track = Closure::wrap(Box::new(move |ev: RtcTrackEvent| {
            if let Ok(streams) = ev.streams().get(0).dyn_into::<MediaStream>() {
                (on_track_cb)(peer_id_clone_2.clone(), streams);
            }
        }) as Box<dyn FnMut(RtcTrackEvent)>);
        pc.set_ontrack(Some(on_track.as_ref().unchecked_ref()));
        on_track.forget();

        // On Negotiation Needed
        // Automatically handle renegotiation
        // let this_clone = self.clone(); // Can't easily clone inside create_peer_connection unless we pass it or structure differently
        // For now, we manually trigger update_local_tracks, so we might skip this unless explicitly requested.
        // But PR comment suggests adding it.
        // Let's implement manual trigger via update_local_tracks for now as it's safer given struct design.

        Ok(pc)
    }

    pub fn handle_participant_joined(&self, peer_id: String) {
        let peers = self.peers.clone();
        let this = self.clone();
        spawn_local(async move {
            if let Ok(pc) = this.create_peer_connection(&peer_id) {
                peers.borrow_mut().insert(peer_id.clone(), pc.clone());

                // Create Offer
                let options = web_sys::RtcOfferOptions::new();
                options.set_offer_to_receive_audio(true);
                options.set_offer_to_receive_video(true);

                let offer_promise = pc.create_offer_with_rtc_offer_options(&options);
                match JsFuture::from(offer_promise).await {
                    Ok(offer) => {
                        let sdp = offer.unchecked_into::<RtcSessionDescriptionInit>();
                        let set_local_promise = pc.set_local_description(&sdp);
                        if JsFuture::from(set_local_promise).await.is_ok() {
                             // Send Offer
                             if let Some(desc) = pc.local_description() {
                                 let sdp_str = desc.sdp();
                                 let msg = shared::ClientMessage::Offer {
                                     target_id: peer_id,
                                     sdp: sdp_str,
                                 };
                                 (this.send_signal)(msg);
                             }
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&e);
                    }
                }
            }
        });
    }

    pub fn handle_participant_left(&self, peer_id: &str) {
        if let Some(pc) = self.peers.borrow_mut().remove(peer_id) {
            pc.close();
        }
    }

    pub fn close_all_peers(&self) {
        let mut peers = self.peers.borrow_mut();
        for (_, pc) in peers.iter() {
            pc.close();
        }
        peers.clear();
    }

    pub fn handle_offer(&self, source_id: String, sdp: String) {
        let peers = self.peers.clone();
        let this = self.clone();
        let my_id = self.my_id.get_untracked();

        spawn_local(async move {
            let pc = if let Some(pc) = peers.borrow().get(&source_id) {
                pc.clone()
            } else if let Ok(pc) = this.create_peer_connection(&source_id) {
                peers.borrow_mut().insert(source_id.clone(), pc.clone());
                pc
            } else {
                return;
            };

            // Perfect Negotiation Glare Handling
            // Politeness: lexicographical comparison of IDs.
            // If my_id < source_id, I am polite (yield).
            // If my_id > source_id, I am impolite (ignore incoming if colliding).

            let is_polite = if let Some(my) = &my_id {
                my.as_str() < source_id.as_str()
            } else {
                true // Fallback to polite if my_id unknown? Or fail?
            };

            let signaling_state = pc.signaling_state();
            // web_sys::RtcSignalingState enum: Stable, HaveLocalOffer, ...
            // Wait, signaling_state() returns RtcSignalingState enum object in JS,
            // but in Rust web-sys it returns RtcSignalingState enum which we can compare?
            // Actually it returns the enum type.

            // Checks for collision: we have sent an offer (have-local-offer) but received one.
            if signaling_state != web_sys::RtcSignalingState::Stable {
                if !is_polite {
                    // Impolite peer ignores the offer
                    return;
                }
                // Polite peer must rollback local offer before accepting remote offer
                let rollback = RtcSessionDescriptionInit::new(RtcSdpType::Rollback);
                if JsFuture::from(pc.set_local_description(&rollback)).await.is_err() {
                    return;
                }
            }

            let desc_init = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
            desc_init.set_sdp(&sdp);

            let set_remote_promise = pc.set_remote_description(&desc_init);
            if JsFuture::from(set_remote_promise).await.is_ok() {
                let create_answer_promise = pc.create_answer();
                if let Ok(answer) = JsFuture::from(create_answer_promise).await {
                    let answer_sdp = answer.unchecked_into::<RtcSessionDescriptionInit>();
                    let set_local_promise = pc.set_local_description(&answer_sdp);
                    if JsFuture::from(set_local_promise).await.is_ok() {
                        if let Some(desc) = pc.local_description() {
                            let sdp_str = desc.sdp();
                            let msg = shared::ClientMessage::Answer {
                                target_id: source_id,
                                sdp: sdp_str,
                            };
                            (this.send_signal)(msg);
                        }
                    }
                }
            }
        });
    }

    pub fn handle_answer(&self, source_id: String, sdp: String) {
        let peers = self.peers.clone();
        spawn_local(async move {
            let pc = peers.borrow().get(&source_id).cloned();
            if let Some(pc) = pc {
                let desc_init = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                desc_init.set_sdp(&sdp);
                let promise = pc.set_remote_description(&desc_init);
                let _ = JsFuture::from(promise).await;
            }
        });
    }

    pub fn handle_ice_candidate(
        &self,
        source_id: String,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    ) {
        let peers = self.peers.clone();
        spawn_local(async move {
            let pc = peers.borrow().get(&source_id).cloned();
            if let Some(pc) = pc {
                let init = RtcIceCandidateInit::new(&candidate);
                if let Some(mid) = sdp_mid {
                    init.set_sdp_mid(Some(&mid));
                }
                if let Some(idx) = sdp_m_line_index {
                    init.set_sdp_m_line_index(Some(idx));
                }

                if let Ok(cand) = RtcIceCandidate::new(&init) {
                    let promise = pc.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&cand));
                    let _ = JsFuture::from(promise).await;
                }
            }
        });
    }

    pub fn has_peer(&self, peer_id: &str) -> bool {
        self.peers.borrow().contains_key(peer_id)
    }

    pub fn update_local_tracks(&self) {
        let peers = self.peers.clone();
        let this = self.clone();

        spawn_local(async move {
            let peer_entries: Vec<(String, RtcPeerConnection)> = {
                let peers_map = peers.borrow();
                peers_map.iter().map(|(id, pc)| (id.clone(), pc.clone())).collect()
            };

            // Iterate over all connected peers
            for (peer_id, pc) in peer_entries {
                let this = this.clone();

                let camera_stream = this.local_stream.get_untracked();
                let screen_stream = this.local_screen_stream.get_untracked();

                // Collect valid track IDs
                let mut valid_track_ids = Vec::new();
                if let Some(s) = &camera_stream {
                    let tracks = s.get_tracks();
                    for track in tracks.iter() {
                         let track = track.unchecked_ref::<web_sys::MediaStreamTrack>();
                         valid_track_ids.push(track.id());
                    }
                }
                if let Some(s) = &screen_stream {
                    let tracks = s.get_tracks();
                    for track in tracks.iter() {
                         let track = track.unchecked_ref::<web_sys::MediaStreamTrack>();
                         valid_track_ids.push(track.id());
                    }
                }

                // Remove invalid senders
                let senders = pc.get_senders();
                for sender in senders.iter() {
                    let sender = sender.unchecked_ref::<web_sys::RtcRtpSender>();
                    if let Some(track) = sender.track() {
                        if !valid_track_ids.contains(&track.id()) {
                             let _ = pc.remove_track(&sender);
                        }
                    }
                }

                // Add missing tracks (Camera)
                if let Some(s) = &camera_stream {
                     let tracks = s.get_tracks();
                     for track in tracks.iter() {
                         let track = track.unchecked_ref::<web_sys::MediaStreamTrack>();
                         // Check if already sending
                         let mut already_sending = false;
                         for sender in senders.iter() {
                             let sender = sender.unchecked_ref::<web_sys::RtcRtpSender>();
                             if let Some(t) = sender.track() {
                                 if t.id() == track.id() {
                                     already_sending = true;
                                     break;
                                 }
                             }
                         }
                         if !already_sending {
                             let streams = js_sys::Array::new();
                             streams.push(s);
                             let _ = pc.add_track(track, s, &streams);
                         }
                     }
                }

                // Add missing tracks (Screen)
                if let Some(s) = &screen_stream {
                     let tracks = s.get_tracks();
                     for track in tracks.iter() {
                         let track = track.unchecked_ref::<web_sys::MediaStreamTrack>();
                         // Check if already sending
                         let mut already_sending = false;
                         for sender in senders.iter() {
                             let sender = sender.unchecked_ref::<web_sys::RtcRtpSender>();
                             if let Some(t) = sender.track() {
                                 if t.id() == track.id() {
                                     already_sending = true;
                                     break;
                                 }
                             }
                         }
                         if !already_sending {
                             let streams = js_sys::Array::new();
                             streams.push(s);
                             let _ = pc.add_track(track, s, &streams);
                         }
                     }
                }

                // Trigger renegotiation (Offer)
                let options = web_sys::RtcOfferOptions::new();
                options.set_offer_to_receive_audio(true);
                options.set_offer_to_receive_video(true);

                if let Ok(offer) = JsFuture::from(pc.create_offer_with_rtc_offer_options(&options)).await {
                    let sdp = offer.unchecked_into::<RtcSessionDescriptionInit>();
                    if JsFuture::from(pc.set_local_description(&sdp)).await.is_ok() {
                        if let Some(desc) = pc.local_description() {
                            let sdp_str = desc.sdp();
                            let msg = shared::ClientMessage::Offer {
                                target_id: peer_id,
                                sdp: sdp_str,
                            };
                            (this.send_signal)(msg);
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_instantiation() {
        let _runtime = create_runtime();
        let (local_stream, _) = create_signal(None);
        let (local_screen_stream, _) = create_signal(None);
        let (my_id, _) = create_signal(Some("me".to_string()));
        let _manager = WebRTCManager::new(
            |_| {},
            |_, _| {},
            local_stream.into(),
            local_screen_stream.into(),
            my_id.into(),
        );
    }
}
