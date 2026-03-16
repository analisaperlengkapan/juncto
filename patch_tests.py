import os
import glob

def main():
    test_dir = "rust-app/tests/e2e"
    files = glob.glob(os.path.join(test_dir, "*.spec.ts"))

    for f in files:
        with open(f, "r") as file:
            content = file.read()

        # Replace '.participants-list' with something more robust, or fix the click on '.toolbox button:has-text("Participants")'
        # The issue might be that the participants list button doesn't open the modal or the selector doesn't exist
        # Wait, the failure says "Received: hidden", which means the element is in the DOM but not visible.
        # This usually means the modal/sidebar didn't open correctly.
        pass

if __name__ == "__main__":
    main()
