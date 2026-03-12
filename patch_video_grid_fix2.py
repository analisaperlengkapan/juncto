with open('rust-app/frontend/src/components_ui/video_grid.rs', 'r') as f:
    content = f.read()

# Fix the remote user speaking indicator
content = content.replace('<Show when=move || speaking_peers.get().contains(&p.id.clone())>\n                                            <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 3px solid #28a745; box-sizing: border-box; border-radius: 8px; pointer-events: none; z-index: 5;"></div>\n                                        </Show>', '<Show when={let p_id = p.id.clone(); move || speaking_peers.get().contains(&p_id)}>\n                                            <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 3px solid #28a745; box-sizing: border-box; border-radius: 8px; pointer-events: none; z-index: 5;"></div>\n                                        </Show>')

with open('rust-app/frontend/src/components_ui/video_grid.rs', 'w') as f:
    f.write(content)
