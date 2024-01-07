import subprocess

files = [
    "src-tauri/src/main.rs",
    "src-tauri/tauri.conf.json",
    "src/routes/+layout.ts",
    "src/routes/+layout.svelte",
    "src/routes/+page.svelte",
    "src/lib/Canvas.svelte",
    "src/lib/index.ts",
    "src/global.css",
]

with open("state.txt", "w") as state_file:
    for filename in files:
        state_file.write(f"--- {filename} ---\n```\n")
        state_file.write(open(filename, "r").read())
        state_file.write("```\n\n")

subprocess.call("pbcopy < state.txt", shell=True)
