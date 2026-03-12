with open('rust-app/frontend/src/components_ui/video_grid.rs', 'r') as f:
    content = f.read()

# Fix the local user speaking indicator
content = content.replace('<Show when=move || speaking_peers.get().contains(&p.id)>\n                    <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 3px solid #28a745; box-sizing: border-box; border-radius: 8px; pointer-events: none; z-index: 5;"></div>\n                </Show>\n                <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">\n                    "Me"', '<Show when=move || speaking_peers.get().contains(&my_id.get().unwrap_or_default())>\n                    <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 3px solid #28a745; box-sizing: border-box; border-radius: 8px; pointer-events: none; z-index: 5;"></div>\n                </Show>\n                <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">\n                    "Me"')

with open('rust-app/frontend/src/components_ui/video_grid.rs', 'w') as f:
    f.write(content)
