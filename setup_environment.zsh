#!/bin/zsh
# setup_environment.zsh
# One-command setup for OpenBB Company Research Tool.

set -e

PROJECT_DIR="${PROJECT_DIR:-$(pwd)}"
COMMAND_NAME="${COMMAND_NAME:-cresearch}"
SECONDARY_COMMAND_NAME="${SECONDARY_COMMAND_NAME:-openbb-research}"
LOCAL_BIN_DIR="${LOCAL_BIN_DIR:-$HOME/.local/bin}"
ZSHRC_FILE="${ZSHRC_FILE:-$HOME/.zshrc}"

echo "== OpenBB Company Research Tool Environment Setup =="
echo "Project dir : $PROJECT_DIR"
echo "Command    : $COMMAND_NAME"
echo "Alias      : $SECONDARY_COMMAND_NAME"
echo ""

cd "$PROJECT_DIR"

if [ ! -f "requirements.txt" ]; then
  echo "ERROR: requirements.txt not found. Run this script in the project root."
  exit 1
fi

if [ ! -f "scripts/company_research_tool.py" ]; then
  echo "ERROR: scripts/company_research_tool.py not found."
  exit 1
fi

if [ ! -d ".venv" ]; then
  echo "Creating Python virtual environment..."
  python3 -m venv .venv
else
  echo ".venv already exists."
fi

source .venv/bin/activate

echo "Upgrading pip..."
python -m pip install --upgrade pip setuptools wheel

echo "Installing dependencies..."
pip install -r requirements.txt

mkdir -p "$LOCAL_BIN_DIR"

cat > "$LOCAL_BIN_DIR/$COMMAND_NAME" <<EOF
#!/bin/zsh
PROJECT_DIR="$PROJECT_DIR"

cd "\$PROJECT_DIR" || exit 1

if [ ! -f ".venv/bin/activate" ]; then
  echo "ERROR: .venv not found in \$PROJECT_DIR"
  exit 1
fi

source .venv/bin/activate
python scripts/company_research_tool.py "\$@"
EOF

chmod +x "$LOCAL_BIN_DIR/$COMMAND_NAME"

if [ "$SECONDARY_COMMAND_NAME" != "$COMMAND_NAME" ]; then
  cat > "$LOCAL_BIN_DIR/$SECONDARY_COMMAND_NAME" <<EOF
#!/bin/zsh
PROJECT_DIR="$PROJECT_DIR"

cd "\$PROJECT_DIR" || exit 1

if [ ! -f ".venv/bin/activate" ]; then
  echo "ERROR: .venv not found in \$PROJECT_DIR"
  exit 1
fi

source .venv/bin/activate
python -m openbb_company_research_tool "\$@"
EOF
  chmod +x "$LOCAL_BIN_DIR/$SECONDARY_COMMAND_NAME"
fi

PATH_LINE='export PATH="$HOME/.local/bin:$PATH"'

if [ ! -f "$ZSHRC_FILE" ]; then
  touch "$ZSHRC_FILE"
fi

if ! grep -Fq "$PATH_LINE" "$ZSHRC_FILE"; then
  cp "$ZSHRC_FILE" "$ZSHRC_FILE.backup.$(date +%Y%m%d%H%M%S)"
  {
    echo ""
    echo "# Local user commands"
    echo "$PATH_LINE"
  } >> "$ZSHRC_FILE"
  echo "Added ~/.local/bin to PATH in ~/.zshrc"
else
  echo "PATH already configured."
fi

echo ""
echo "Testing command..."
"$LOCAL_BIN_DIR/$COMMAND_NAME" --help >/tmp/cresearch_setup_test.txt 2>&1 || {
  echo "ERROR: command test failed."
  cat /tmp/cresearch_setup_test.txt
  exit 1
}

if [ "$SECONDARY_COMMAND_NAME" != "$COMMAND_NAME" ]; then
  "$LOCAL_BIN_DIR/$SECONDARY_COMMAND_NAME" --help >/tmp/openbb_research_setup_test.txt 2>&1 || {
    echo "ERROR: secondary command test failed."
    cat /tmp/openbb_research_setup_test.txt
    exit 1
  }
fi

echo "Setup complete."
echo ""
echo "Run:"
echo "  source ~/.zshrc"
echo ""
echo "Then try:"
echo "  $COMMAND_NAME --help"
echo "  $SECONDARY_COMMAND_NAME --help"
echo "  $COMMAND_NAME AAPL --benchmark SPY --start 2023-01-01"
