import os
import sys
import platform
import urllib.request
import stat

VERSION = "0.0.1-beta-3"
GITHUB_REPO = "peekstore/tasklynx"

def get_binary_name():
    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "linux":
        os_name = "linux"
    elif system == "darwin":
        os_name = "darwin"
    elif system == "windows":
        os_name = "windows"
    else:
        raise RuntimeError(f"Unsupported OS: {system}")

    if machine in ("x86_64", "amd64"):
        arch_name = "amd64"
    elif machine in ("aarch64", "arm64"):
        arch_name = "arm64"
    elif machine.startswith("arm"):
        arch_name = "arm64"
    else:
        raise RuntimeError(f"Unsupported architecture: {machine}")

    ext = ".exe" if os_name == "windows" else ""
    return f"tasklynx-{os_name}-{arch_name}{ext}"

def get_cache_path():
    home = os.environ.get("TASKLYNX_HOME")
    if home:
        cache_dir = os.path.join(home, "bin")
    else:
        cache_dir = os.path.expanduser("~/.local/share/tasklynx/bin")
    os.makedirs(cache_dir, exist_ok=True)
    binary_name = get_binary_name()
    return os.path.join(cache_dir, f"tasklynx-v{VERSION}-{binary_name}")

def ensure_binary():
    bin_path = get_cache_path()
    if os.path.exists(bin_path) and os.path.getsize(bin_path) > 0:
        return bin_path

    binary_name = get_binary_name()
    url = f"https://github.com/{GITHUB_REPO}/releases/download/v{VERSION}/{binary_name}"
    print(f"Downloading tasklynx v{VERSION} for {platform.system()}-{platform.machine()}...", file=sys.stderr)

    try:
        urllib.request.urlretrieve(url, bin_path)
    except Exception as e:
        if os.path.exists(bin_path):
            os.remove(bin_path)
        raise RuntimeError(f"Failed to download tasklynx from {url}: {e}")

    # Set executable permissions
    current_mode = os.stat(bin_path).st_mode
    os.chmod(bin_path, current_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return bin_path

def main():
    try:
        binary = ensure_binary()
    except Exception as err:
        print(f"Error: {err}", file=sys.stderr)
        sys.exit(1)

    if platform.system() == "Windows":
        import subprocess
        result = subprocess.run([binary] + sys.argv[1:])
        sys.exit(result.returncode)
    else:
        os.execvp(binary, [binary] + sys.argv[1:])

if __name__ == "__main__":
    main()
