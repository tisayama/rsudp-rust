#!/bin/bash
set -e

# T002: Setup Python venv
echo "Setting up Python environment..."
python3 -m venv rsudp-venv
source rsudp-venv/bin/activate

# Upgrade pip
pip install --upgrade pip

# Installing dependencies from references/rsudp/docsrc/requirements.txt...
pip install -r references/rsudp/docsrc/requirements.txt

# Additional dependencies required by updated rsudp modules
pip install boto3 botocore line-bot-sdk atproto

echo "Python environment setup complete."
