import os
import glob

def main():
    test_dir = "rust-app/tests/e2e"
    files = glob.glob(os.path.join(test_dir, "*.spec.ts"))

    for f in files:
        with open(f, "r") as file:
            content = file.read()

        # In Kick Participant, it tries to click Kick but it's outside the viewport (or maybe inside a hidden scroll container).
        # We can force the click and ensure the participants list is open.

        if "Kick Participant E2E" in content:
            # We already patched it to conditionally open. Let's verify it actually does.
            pass

if __name__ == "__main__":
    main()
