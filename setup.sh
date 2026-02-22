#!/usr/bin/env bash
set -e

# lrc-tools setup

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/lrc-tools"
PKG_DIR="$CONFIG_DIR/lrc_tools"          # actual package lives here
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/lrc-tools"
BIN_DIR="$HOME/.local/bin"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=============================="
echo " lrc-tools setup"
echo "=============================="
echo "Package:    $PKG_DIR"
echo "Data:       $DATA_DIR"
echo "Bins:       $BIN_DIR"
echo

# ---------------------------------------------------------------------------
# Python deps
# ---------------------------------------------------------------------------
echo "[1/4] Checking Python dependencies..."

check_pip() {
    python3 -c "import $1" 2>/dev/null && echo "  ✓ $1" || {
        echo "  installing $1..."
        pip install "$2" --break-system-packages --quiet 2>/dev/null \
            || pip install "$2" --user --quiet
    }
}

check_pip yaml pyyaml
check_pip mutagen mutagen
check_pip syncedlyrics syncedlyrics

echo
read -rp "Install librosa for onset detection (optional, ~200MB)? [y/N] " librosa_ans
if [[ "${librosa_ans,,}" == "y" ]]; then
    pip install librosa --break-system-packages --quiet 2>/dev/null \
        || pip install librosa --user --quiet
    echo "  ✓ librosa"
fi

# ---------------------------------------------------------------------------
# System deps
# ---------------------------------------------------------------------------
echo
echo "[2/4] Checking system dependencies..."

check_cmd() {
    command -v "$1" &>/dev/null && echo "  ✓ $1" \
        || echo "  ✗ $1 not found — install with: $2"
}

check_cmd ffprobe "sudo pacman -S ffmpeg"
check_cmd playerctl "sudo pacman -S playerctl"

# ---------------------------------------------------------------------------
# Install package into ~/.config/lrc-tools/lrc_tools/
# ---------------------------------------------------------------------------
echo
echo "[3/4] Installing package..."

mkdir -p "$PKG_DIR"

if [[ -d "$SCRIPT_DIR/lrc_tools" ]]; then
    cp -r "$SCRIPT_DIR/lrc_tools/". "$PKG_DIR/"
else
    # flat layout — copy all .py files into the package dir
    cp "$SCRIPT_DIR"/*.py "$PKG_DIR/"
fi

echo "  ✓ package → $PKG_DIR"

# Config files stay in ~/.config/lrc-tools/ (not the package subdir)
if [[ ! -f "$CONFIG_DIR/config.yaml" ]]; then
    cp "$SCRIPT_DIR/config_example.yaml" "$CONFIG_DIR/config.yaml"
    echo "  ✓ config.yaml → $CONFIG_DIR"
else
    echo "  ✓ config.yaml already exists, skipping"
fi

if [[ ! -f "$CONFIG_DIR/custom_fonts.json" ]]; then
    cp "$SCRIPT_DIR/custom_fonts.json" "$CONFIG_DIR/custom_fonts.json"
    echo "  ✓ custom_fonts.json → $CONFIG_DIR"
else
    echo "  ✓ custom_fonts.json already exists, skipping"
fi

# ---------------------------------------------------------------------------
# Bin stubs — add ~/.config/lrc-tools to sys.path so lrc_tools is importable
# ---------------------------------------------------------------------------
mkdir -p "$BIN_DIR"

write_stub() {
    local bin_name="$1"
    local cli_module="$2"
    local dest="$BIN_DIR/$bin_name"

    cat > "$dest" << EOF
#!/usr/bin/env python3
import sys
sys.path.insert(0, '$CONFIG_DIR')
from lrc_tools.$cli_module import main
sys.exit(main())
EOF
    chmod +x "$dest"
    echo "  ✓ $bin_name → $dest"
}

write_stub lrc-fetch      lrc_puller_cli
write_stub lrc-processor  lrc_processor_cli
write_stub lrc-vis        lrc_vis_cli

# Warn if ~/.local/bin not on PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo
    echo "  ⚠ $BIN_DIR is not in your PATH"
    echo "  Add to ~/.config/fish/config.fish:"
    echo "    fish_add_path $BIN_DIR"
fi

# ---------------------------------------------------------------------------
# Data dirs
# ---------------------------------------------------------------------------
echo
echo "[4/4] Creating data directories..."

mkdir -p "$DATA_DIR/lyrics/raw"
mkdir -p "$DATA_DIR/lyrics/processed"
echo "  ✓ $DATA_DIR/lyrics/{raw,processed}"

echo
echo "=============================="
echo " Done!"
echo "=============================="
echo
echo "Quickstart:"
echo
echo "  lrc-fetch --audio-dir ~/music --output-dir $DATA_DIR/lyrics/raw"
echo
echo "  lrc-processor --lrc-dir $DATA_DIR/lyrics/raw \\"
echo "                --audio-dir ~/music \\"
echo "                --output-dir $DATA_DIR/lyrics/processed \\"
echo "                --wlrc"
echo
echo "  lrc-vis --lrc-dir $DATA_DIR/lyrics/processed --wlrc"
