import os

react_dir = 'react/features'
rust_dir = 'rust-app/frontend/src'

react_features = set()
if os.path.exists(react_dir):
    react_features = set([d for d in os.listdir(react_dir) if os.path.isdir(os.path.join(react_dir, d))])

# These might be in different places in rust, let's just make a list of features that are generally expected
# based on the legacy codebase that we haven't seen in the Rust codebase yet.
# 'audio-level-indicator'
# 'remote-control'
# 'pip' (exists somewhat in video grid, but maybe needs dedicated component?)
# 'no-audio-signal'
# 'face-landmarks'
# 'embed-meeting'
