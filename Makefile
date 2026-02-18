# Makefile for rsudp-rust

# Variables
BINARY_NAME := rsudp-rust
SERVICE_NAME := rsudp
CARGO_TOML := rsudp-rust/Cargo.toml
TARGET_RELEASE := rsudp-rust/target/release/$(BINARY_NAME)
INSTALL_BIN_DIR := /usr/local/bin
INSTALL_CONF_DIR := /etc/rsudp
INSTALL_DATA_DIR := /var/lib/rsudp
SYSTEMD_DIR := /etc/systemd/system
SERVICE_FILE := rsudp-rust/systemd/$(SERVICE_NAME).service
CAPTURE_SERVICE_NAME := rsudp-capture
CAPTURE_INSTALL_DIR := /opt/rsudp-capture
CAPTURE_SERVICE_FILE := rsudp-rust/systemd/$(CAPTURE_SERVICE_NAME).service
WEBUI_SERVICE_NAME := rsudp-webui
WEBUI_INSTALL_DIR := /opt/rsudp-webui
WEBUI_SERVICE_FILE := rsudp-rust/systemd/$(WEBUI_SERVICE_NAME).service
USER := rsudp
GROUP := rsudp

.PHONY: all build install install-deps install-capture install-webui setup-user uninstall clean

all: build

# Install system-level prerequisites (requires root)
install-deps:
	@echo "Installing system dependencies..."
	apt-get update
	apt-get install -y libssl-dev nodejs npm libgtk-4-dev libgraphene-1.0-dev
	@echo "System dependencies installed."

# Create rsudp system user and group
setup-user:
	@echo "Setting up $(USER) user..."
	getent group $(GROUP) >/dev/null || groupadd -r $(GROUP)
	id -u $(USER) &>/dev/null || useradd -r -g $(GROUP) -G audio -d $(INSTALL_DATA_DIR) -s /usr/sbin/nologin -c "rsudp service account" $(USER)
	@echo "User $(USER):$(GROUP) ready."

build:
	@echo "Building $(BINARY_NAME)..."
	cargo build --release --manifest-path $(CARGO_TOML)

install: install-deps setup-user install-webui install-capture
	@echo "Installing $(BINARY_NAME)..."

	# Install binary
	install -m 755 $(TARGET_RELEASE) $(INSTALL_BIN_DIR)/$(BINARY_NAME)

	# Install config
	mkdir -p $(INSTALL_CONF_DIR)
	[ -f $(INSTALL_CONF_DIR)/rsudp.toml ] || install -m 640 -o root -g $(GROUP) rsudp-rust/rsudp_settings.toml $(INSTALL_CONF_DIR)/rsudp.toml

	# Create data directory
	mkdir -p $(INSTALL_DATA_DIR)
	chown $(USER):$(GROUP) $(INSTALL_DATA_DIR)
	chmod 750 $(INSTALL_DATA_DIR)

	# Install sound files
	install -d -o $(USER) -g $(GROUP) $(INSTALL_DATA_DIR)/sounds
	install -m 644 -o $(USER) -g $(GROUP) rsudp-rust/sounds/*.mp3 $(INSTALL_DATA_DIR)/sounds/

	# Install systemd service
	install -m 644 $(SERVICE_FILE) $(SYSTEMD_DIR)/$(SERVICE_NAME).service

	# Reload systemd
	systemctl daemon-reload

	@echo "Installation complete. Run 'systemctl start $(SERVICE_NAME)' to start the service."

install-webui: setup-user
	@echo "Installing WebUI..."
	# Build WebUI
	cd webui && npm ci && npm run build

	# Install standalone output
	install -d $(WEBUI_INSTALL_DIR)
	cp -r webui/.next/standalone/. $(WEBUI_INSTALL_DIR)/
	cp -r webui/.next/static $(WEBUI_INSTALL_DIR)/.next/static
	cp -r webui/public $(WEBUI_INSTALL_DIR)/public

	# Set ownership
	chown -R $(USER):$(GROUP) $(WEBUI_INSTALL_DIR)

	# Install systemd service
	install -m 644 $(WEBUI_SERVICE_FILE) $(SYSTEMD_DIR)/$(WEBUI_SERVICE_NAME).service

	# Reload systemd
	systemctl daemon-reload

	@echo "WebUI installed. Run 'systemctl start $(WEBUI_SERVICE_NAME)' to start."

install-capture: setup-user
	@echo "Installing capture service..."
	# Create capture service directory
	install -d $(CAPTURE_INSTALL_DIR)

	# Copy capture service files
	install -m 644 capture-service/server.js $(CAPTURE_INSTALL_DIR)/server.js
	install -m 644 capture-service/package.json $(CAPTURE_INSTALL_DIR)/package.json

	# Install Node.js dependencies
	cd $(CAPTURE_INSTALL_DIR) && npm install --production

	# Set ownership before Playwright install (rsudp user needs write access)
	chown -R $(USER):$(GROUP) $(CAPTURE_INSTALL_DIR)

	# Install Playwright system dependencies (as root)
	cd $(CAPTURE_INSTALL_DIR) && npx playwright install-deps chromium

	# Install Playwright Chromium browser (as rsudp user)
	su -s /bin/sh $(USER) -c 'cd $(CAPTURE_INSTALL_DIR) && npx playwright install chromium'

	# Install systemd service
	install -m 644 $(CAPTURE_SERVICE_FILE) $(SYSTEMD_DIR)/$(CAPTURE_SERVICE_NAME).service

	# Reload systemd
	systemctl daemon-reload

	@echo "Capture service installed. Run 'systemctl start $(CAPTURE_SERVICE_NAME)' to start."

uninstall:
	@echo "Uninstalling $(BINARY_NAME)..."
	systemctl stop $(SERVICE_NAME) || true
	systemctl disable $(SERVICE_NAME) || true
	rm -f $(INSTALL_BIN_DIR)/$(BINARY_NAME)
	rm -f $(SYSTEMD_DIR)/$(SERVICE_NAME).service
	systemctl daemon-reload
	@echo "Uninstallation complete. Data and config files were preserved."

clean:
	@echo "Cleaning up..."
	cargo clean --manifest-path $(CARGO_TOML)