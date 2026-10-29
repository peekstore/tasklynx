import os
import sys
import platform
import stat
import subprocess
import urllib.request
import urllib.error

VERSION = "0.0.1-beta-4"
GITHUB_REPO = "peekstore/tasklynx"
MIN_BINARY_SIZE = 100 * 1024  # 100 KB sanity check

# ---------------------------------------------------------------------------
# Platform / architecture detection
# ---------------------------------------------------------------------------

def is_termux() -> bool:
    """Return True if running inside Android Termux."""
    if os.environ.get("TERMUX_VERSION"):
        return True
    return os.path.isdir("/data/data/com.termux")


def _termux_abi() -> str:
    """
    Detect the Android ABI via `getprop ro.product.cpu.abi`.
    Falls back to platform.machine() when getprop is unavailable.
    """
    try:
        result = subprocess.run(
            ["getprop", "ro.product.cpu.abi"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        abi = result.stdout.strip().lower()
    except Exception:
        abi = platform.machine().lower()

    if abi in ("arm64-v8a", "aarch64", "arm64"):
        return "arm64-v8a"
    elif abi in ("armeabi-v7a", "armv7l", "arm"):
        return "armeabi-v7a"
    elif abi in ("x86_64", "x86-64", "amd64"):
        return "x86_64"
    else:
        raise RuntimeError(
            f"Unsupported Android ABI: {abi!r}. "
            "Supported ABIs: arm64-v8a, armeabi-v7a, x86_64."
        )


def get_binary_name() -> str:
    """
    Return the bare release-asset filename, e.g.:
      tasklynx-linux-amd64
      tasklynx-darwin-arm64
      tasklynx-windows-amd64.exe
      tasklynx-android-arm64-v8a
    """
    if is_termux():
        abi = _termux_abi()
        return f"tasklynx-android-{abi}"

    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "linux":
        os_name = "linux"
    elif system == "darwin":
        os_name = "darwin"
    elif system == "windows":
        os_name = "windows"
    else:
        raise RuntimeError(
            f"Unsupported OS: {system!r}. "
            "Supported: linux, darwin, windows, Android/Termux."
        )

    if machine in ("x86_64", "amd64"):
        arch_name = "amd64"
    elif machine in ("aarch64", "arm64"):
        arch_name = "arm64"
    elif machine.startswith("arm"):
        arch_name = "arm64"  # best-effort fallback
    else:
        raise RuntimeError(
            f"Unsupported architecture: {machine!r} on {os_name}. "
            "Supported: x86_64/amd64, aarch64/arm64."
        )

    ext = ".exe" if os_name == "windows" else ""
    return f"tasklynx-{os_name}-{arch_name}{ext}"


# ---------------------------------------------------------------------------
# Cache / install directory  (~/.config/peekstore/tasklynx/bin)
# ---------------------------------------------------------------------------

def get_install_dir() -> str:
    """
    Returns the binary install directory, consistent with installer.sh:
      Linux / macOS / Android: $XDG_CONFIG_HOME/peekstore/tasklynx/bin
                                (defaults to ~/.config/peekstore/tasklynx/bin)
      Windows:                 %APPDATA%\\peekstore\\tasklynx\\bin
    """
    _platform = platform.system().lower()

    if is_termux() or _platform in ("linux", "darwin"):
        xdg = os.environ.get("XDG_CONFIG_HOME") or os.path.join(os.path.expanduser("~"), ".config")
        return os.path.join(xdg, "peekstore", "tasklynx", "bin")

    if _platform == "windows":
        app_data = os.environ.get("APPDATA") or os.path.join(os.path.expanduser("~"), "AppData", "Roaming")
        return os.path.join(app_data, "peekstore", "tasklynx", "bin")

    # Fallback
    return os.path.join(os.path.expanduser("~"), ".config", "peekstore", "tasklynx", "bin")


def get_bin_path() -> str:
    install_dir = get_install_dir()
    os.makedirs(install_dir, exist_ok=True)
    is_win = platform.system().lower() == "windows"
    return os.path.join(install_dir, "tasklynx.exe" if is_win else "tasklynx")


def get_tx_path() -> str:
    install_dir = get_install_dir()
    is_win = platform.system().lower() == "windows"
    return os.path.join(install_dir, "tx.exe" if is_win else "tx")


# ---------------------------------------------------------------------------
# Download with progress display
# ---------------------------------------------------------------------------

def _progress_hook(count: int, block_size: int, total_size: int) -> None:
    received = count * block_size
    if total_size > 0:
        pct = min(100, int(received * 100 / total_size))
        bar = "#" * (pct // 2)
        sys.stderr.write(f"\r  [{bar:<50}] {pct:3d}%  {_fmt(received)}/{_fmt(total_size)}  ")
    else:
        sys.stderr.write(f"\r  Downloaded: {_fmt(received)}  ")
    sys.stderr.flush()


def _fmt(b: int) -> str:
    if b < 1024:
        return f"{b} B"
    if b < 1024 ** 2:
        return f"{b / 1024:.1f} KB"
    return f"{b / (1024 ** 2):.2f} MB"


def download_binary(url: str, dest: str) -> int:
    """
    Download *url* to *dest* with a progress bar.
    Returns the final file size in bytes.
    Raises RuntimeError on HTTP or network errors.
    """
    try:
        urllib.request.urlretrieve(url, dest, reporthook=_progress_hook)
    except urllib.error.HTTPError as exc:
        _cleanup(dest)
        raise RuntimeError(
            f"HTTP {exc.code} downloading binary from:\n  {url}\n"
            "Check that the release exists and the version is correct."
        ) from exc
    except urllib.error.URLError as exc:
        _cleanup(dest)
        raise RuntimeError(
            f"Network error while downloading binary: {exc.reason}"
        ) from exc
    except Exception as exc:
        _cleanup(dest)
        raise RuntimeError(f"Failed to download binary: {exc}") from exc
    finally:
        sys.stderr.write("\n")

    return os.path.getsize(dest)


def _cleanup(path: str) -> None:
    try:
        if os.path.exists(path):
            os.remove(path)
    except OSError:
        pass


# ---------------------------------------------------------------------------
# tx alias helper
# ---------------------------------------------------------------------------

def ensure_tx_alias(bin_path: str) -> None:
    """Create a 'tx' symlink (or copy on Windows) next to the binary."""
    tx_path = get_tx_path()
    is_win = platform.system().lower() == "windows"
    try:
        if os.path.exists(tx_path) or os.path.islink(tx_path):
            os.remove(tx_path)
        if is_win:
            import shutil
            shutil.copy2(bin_path, tx_path)
        else:
            os.symlink(bin_path, tx_path)
    except Exception as exc:
        # tx alias is non-fatal
        sys.stderr.write(f"Warning: could not create tx alias at {tx_path}: {exc}\n")


# ---------------------------------------------------------------------------
# Self-update
# ---------------------------------------------------------------------------

def self_update() -> None:
    """Delete cached binary so the next invocation re-downloads it."""
    bin_path = get_bin_path()
    tx_path = get_tx_path()
    deleted = False
    for p in (bin_path, tx_path):
        if os.path.exists(p) or os.path.islink(p):
            os.remove(p)
            deleted = True
    if deleted:
        sys.stderr.write(
            f"Removed cached binaries. Re-run tasklynx to download v{VERSION} fresh.\n"
        )
    else:
        sys.stderr.write(
            f"No cached binary found (already clean). Run tasklynx to download v{VERSION}.\n"
        )
    sys.exit(0)


# ---------------------------------------------------------------------------
# Ensure binary is present, downloading if needed
# ---------------------------------------------------------------------------

def ensure_binary() -> str:
    bin_path = get_bin_path()
    exists = os.path.exists(bin_path)
    big_enough = exists and os.path.getsize(bin_path) >= MIN_BINARY_SIZE

    if exists and big_enough:
        return bin_path

    binary_name = get_binary_name()
    url = f"https://github.com/{GITHUB_REPO}/releases/download/v{VERSION}/{binary_name}"

    sys.stderr.write(
        f"Downloading tasklynx v{VERSION} ({binary_name})...\n"
        f"  From: {url}\n"
        f"  To:   {bin_path}\n"
    )

    size = download_binary(url, bin_path)

    if size < MIN_BINARY_SIZE:
        _cleanup(bin_path)
        raise RuntimeError(
            f"Downloaded file is too small ({_fmt(size)}). "
            "The release binary may not exist for this platform, or the URL is incorrect."
        )

    # Set executable bit (no-op on Windows)
    if platform.system().lower() != "windows":
        current_mode = os.stat(bin_path).st_mode
        os.chmod(bin_path, current_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    sys.stderr.write(f"Download complete ({_fmt(size)}).\n")
    ensure_tx_alias(bin_path)
    return bin_path


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> None:
    args = sys.argv[1:]

    # --version / -V (no binary needed)
    if "--version" in args or "-V" in args:
        print(f"tasklynx v{VERSION}")
        sys.exit(0)

    # self update
    if args == ["self", "update"] or "--self-update" in args:
        self_update()  # exits

    try:
        binary = ensure_binary()
    except Exception as exc:
        sys.stderr.write(f"Error: {exc}\n")
        sys.exit(1)

    if platform.system().lower() == "windows":
        result = subprocess.run([binary] + args)
        sys.exit(result.returncode)
    else:
        os.execvp(binary, [binary] + args)


if __name__ == "__main__":
    main()
