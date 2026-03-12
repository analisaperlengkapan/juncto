with open('rust-app/frontend/src/components_ui/video_grid.rs', 'r') as f:
    content = f.read()

# Add speaking indicator to Remote Users
indicator = """
                                    <Show when=move || p.presence == shared::PresenceStatus::Connected>
                                        <Show when=move || is_speaking.get()>
                                            <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 3px solid #28a745; box-sizing: border-box; border-radius: 8px; pointer-events: none; z-index: 5;"></div>
                                        </Show>
                                    </Show>
"""

content = content.replace('                                    <div class="status-icons" style="position: absolute; top: 10px; right: 10px; display: flex; gap: 5px;">', indicator + '                                    <div class="status-icons" style="position: absolute; top: 10px; right: 10px; display: flex; gap: 5px;">')

with open('rust-app/frontend/src/components_ui/video_grid.rs', 'w') as f:
    f.write(content)
