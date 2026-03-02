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
    pending_candidates: Rc<RefCell<HashMap<PeerId, Vec<RtcIceCandidateInit>>>>,
    making_offer: Rc<RefCell<HashMap<PeerId, bool>>>,
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
            pending_candidates: Rc::new(RefCell::new(HashMap::new())),
            making_offer: Rc::new(RefCell::new(HashMap::new())),
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
        let pc_clone = pc.clone();
        let making_offer_clone = self.making_offer.clone();
        let peer_id_clone_3 = peer_id.to_string();
        let send_signal_clone = self.send_signal.clone();

        let on_negotiation_needed = Closure::wrap(Box::new(move || {
            let pc = pc_clone.clone();
            let making_offer = making_offer_clone.clone();
            let peer_id = peer_id_clone_3.clone();
            let send_signal = send_signal_clone.clone();

            spawn_local(async move {
                making_offer.borrow_mut().insert(peer_id.clone(), true);

                // Create a block so we can catch any errors and clean up properly
                // mimicking a try/finally block
                let result: Result<(), JsValue> = async {
                    let options = web_sys::RtcOfferOptions::new();
                    options.set_offer_to_receive_audio(true);
                    options.set_offer_to_receive_video(true);

                    // Create Offer
                    let offer = JsFuture::from(pc.create_offer_with_rtc_offer_options(&options)).await?;
                    let sdp = offer.unchecked_into::<RtcSessionDescriptionInit>();

                    // Set Local Description
                    JsFuture::from(pc.set_local_description(&sdp)).await?;

                    if let Some(desc) = pc.local_description() {
                        let sdp_str = desc.sdp();
                        let msg = shared::ClientMessage::Offer {
                            target_id: peer_id.clone(),
                            sdp: sdp_str,
                        };
                        (send_signal)(msg);
                    }
                    Ok(())
                }.await;

                if let Err(e) = result {
                    web_sys::console::error_1(&e);
                }

                // Cleanup (finally)
                making_offer.borrow_mut().insert(peer_id, false);
            });
        }) as Box<dyn FnMut()>);
        pc.set_onnegotiationneeded(Some(on_negotiation_needed.as_ref().unchecked_ref()));
        on_negotiation_needed.forget();

        Ok(pc)
    }

    pub fn handle_participant_joined(&self, peer_id: String) {
        let peers = self.peers.clone();
        let this = self.clone();
        let _pending_candidates = self.pending_candidates.clone();

        spawn_local(async move {
            if let Ok(pc) = this.create_peer_connection(&peer_id) {
                // Ensure we close any existing connection before overwriting
                if let Some(old_pc) = peers.borrow_mut().insert(peer_id.clone(), pc.clone()) {
                    old_pc.close();
                }

                // If remote description already pending (unlikely in 'joined' but good hygiene)
                // Actually in 'joined' we are initiating usually, or expecting them to initiate.
                // If we are polite/impolite logic decides.
                // But pending candidates might exist from early ICE messages

                // Flush pending candidates if we are somehow ready?
                // No, we can only add candidates after remote description is set.
                // But wait, if we are the Offerer, we set remote description later (Answer).
                // So candidates wait.

                // create_peer_connection attaches onnegotiationneeded which will fire automatically
                // because we added tracks inside it.
                // So we do NOT need to manually create offer here anymore!
                // The browser will fire 'negotiationneeded' and our handler will create the offer.
            }
        });
    }

    pub fn handle_participant_left(&self, peer_id: &str) {
        if let Some(pc) = self.peers.borrow_mut().remove(peer_id) {
            pc.close();
        }
        self.pending_candidates.borrow_mut().remove(peer_id);
        self.making_offer.borrow_mut().remove(peer_id);
    }

    pub fn close_all_peers(&self) {
        let mut peers = self.peers.borrow_mut();
        for (_, pc) in peers.iter() {
            pc.close();
        }
        peers.clear();
        self.pending_candidates.borrow_mut().clear();
        self.making_offer.borrow_mut().clear();
    }

    pub fn handle_offer(&self, source_id: String, sdp: String) {
        let peers = self.peers.clone();
        let this = self.clone();
        let my_id = self.my_id.get_untracked();
        let pending_candidates = self.pending_candidates.clone();
        let making_offer_map = self.making_offer.clone();
        // ignore_offer_map is intentionally removed to avoid persisting the flag

        spawn_local(async move {
            // Ensure we use the existing or create new PC.
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
                true // Fallback to polite
            };

            let signaling_state = pc.signaling_state();
            let making_offer = making_offer_map.borrow().get(&source_id).cloned().unwrap_or(false);

            let offer_collision = making_offer || signaling_state != web_sys::RtcSignalingState::Stable;

            if offer_collision {
                if !is_polite {
                    // Impolite peer ignores the incoming offer during collision.
                    // We simply return, allowing our outbound offer to proceed.
                    // The other (polite) peer will rollback and process our offer.
                    return;
                }

                // If polite, we accept the offer.
                // If we are HaveLocalOffer, we must rollback to accept the new offer.
                if signaling_state == web_sys::RtcSignalingState::HaveLocalOffer {
                     let rollback = RtcSessionDescriptionInit::new(RtcSdpType::Rollback);
                     let _ = JsFuture::from(pc.set_local_description(&rollback)).await;
                }
            }

            let desc_init = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
            desc_init.set_sdp(&sdp);

            let set_remote_promise = pc.set_remote_description(&desc_init);
            if JsFuture::from(set_remote_promise).await.is_ok() {
                // Flush pending candidates
                if let Some(candidates) = pending_candidates.borrow_mut().remove(&source_id) {
                    for cand in candidates {
                        if let Ok(rtc_cand) = RtcIceCandidate::new(&cand) {
                            let _ = JsFuture::from(pc.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&rtc_cand))).await;
                        }
                    }
                }

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
        let pending_candidates = self.pending_candidates.clone();

        spawn_local(async move {
            let pc = peers.borrow().get(&source_id).cloned();
            if let Some(pc) = pc {
                let desc_init = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                desc_init.set_sdp(&sdp);
                let promise = pc.set_remote_description(&desc_init);
                if JsFuture::from(promise).await.is_ok() {
                    // Flush pending candidates
                    if let Some(candidates) = pending_candidates.borrow_mut().remove(&source_id) {
                        for cand in candidates {
                            if let Ok(rtc_cand) = RtcIceCandidate::new(&cand) {
                                let _ = JsFuture::from(pc.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&rtc_cand))).await;
                            }
                        }
                    }
                }
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
        let pending_candidates = self.pending_candidates.clone();

        spawn_local(async move {
            // Address Bug 2: Queue candidates even if peer doesn't exist yet
            let init = RtcIceCandidateInit::new(&candidate);
            if let Some(mid) = sdp_mid {
                init.set_sdp_mid(Some(&mid));
            }
            if let Some(idx) = sdp_m_line_index {
                init.set_sdp_m_line_index(Some(idx));
            }

            let pc_opt = peers.borrow().get(&source_id).cloned();

            if let Some(pc) = pc_opt {
                if pc.remote_description().is_some() {
                    if let Ok(cand) = RtcIceCandidate::new(&init) {
                        let promise = pc.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&cand));
                        let _ = JsFuture::from(promise).await;
                    }
                } else {
                    pending_candidates.borrow_mut()
                        .entry(source_id)
                        .or_insert_with(Vec::new)
                        .push(init);
                }
            } else {
                // Bug 2 Fix: Queue even if PC is None
                pending_candidates.borrow_mut()
                    .entry(source_id)
                    .or_insert_with(Vec::new)
                    .push(init);
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
            for (_peer_id, pc) in peer_entries {
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

                // Fetch an updated snapshot of senders after removal to ensure already_sending checks are accurate
                let current_senders = pc.get_senders();

                // Add missing tracks (Camera)
                if let Some(s) = &camera_stream {
                     let tracks = s.get_tracks();
                     for track in tracks.iter() {
                         let track = track.unchecked_ref::<web_sys::MediaStreamTrack>();
                         // Check if already sending
                         let mut already_sending = false;
                         for sender in current_senders.iter() {
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

                // Removed manual offer creation logic.
                // onnegotiationneeded handler (added in create_peer_connection) will trigger
                // because we added/removed tracks above.
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
