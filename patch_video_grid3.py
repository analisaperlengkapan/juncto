with open('rust-app/frontend/src/components_ui/video_grid.rs', 'r') as f:
    content = f.read()

# Replace is_speaking with the correct variables
content = content.replace('<Show when=move || is_speaking.get()>', '<Show when=move || speaking_peers.get().contains(&p.id)>', 1)
content = content.replace('<Show when=move || is_speaking.get()>', '<Show when=move || speaking_peers.get().contains(&my_id.get().unwrap_or_default())>', 1)

with open('rust-app/frontend/src/components_ui/video_grid.rs', 'w') as f:
    f.write(content)
