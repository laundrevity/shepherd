import subprocess
import platform

# List of files to include in the state
files = [
    "src-tauri/src/main.rs",
    "src-tauri/src/game.rs",
    "src-tauri/src/sprites.rs",
    "src-tauri/src/traits.rs",
    "src-tauri/src/constants.rs",
    "src-tauri/src/collision.rs",
    "src-tauri/tauri.conf.json",
    "src/routes/+layout.ts",
    "src/routes/+layout.svelte",
    "src/routes/+page.svelte",
    "src/lib/Canvas.svelte",
    "src/lib/Explosion.svelte",
    "src/lib/index.ts",
    "src/global.css",
]

# Create a file to store the state
with open("state.txt", "w") as state_file:
    for filename in files:
        state_file.write(f"--- {filename} ---\n```\n")
        state_file.write(open(filename, "r").read())
        state_file.write("```\n\n")

# Determine the clipboard command based on the operating system
clipboard_command = ""
if platform.system() == "Darwin":  # macOS
    clipboard_command = "pbcopy"
elif platform.system() == "Linux":  # Linux (assuming xclip is installed)
    clipboard_command = "xclip -selection clipboard"
else:
    print("Unsupported operating system")
    exit(1)

# Copy the contents of state.txt to the clipboard using the appropriate command
subprocess.call(f"{clipboard_command} < state.txt", shell=True)
