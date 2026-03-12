with open('rust-app/frontend/src/components_ui/video_grid.rs', 'r') as f:
    content = f.read()

# Make sure speaking indicator is applied for local user
indicator_local = """
                <Show when=move || is_speaking.get()>
                    <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; border: 3px solid #28a745; box-sizing: border-box; border-radius: 8px; pointer-events: none; z-index: 5;"></div>
                </Show>
"""

content = content.replace('                <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">\n                    "Me"\n                </div>\n            </div>', indicator_local + '                <div class="name-tag" style="position: absolute; bottom: 10px; left: 10px; background: rgba(0,0,0,0.5); color: white; padding: 4px 8px; border-radius: 4px;">\n                    "Me"\n                </div>\n            </div>')

with open('rust-app/frontend/src/components_ui/video_grid.rs', 'w') as f:
    f.write(content)
