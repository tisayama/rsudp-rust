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
USER := rsudp
GROUP := rsudp

.PHONY: all build install uninstall clean

all: build

build:
	@echo "Building $(BINARY_NAME)..."
	cargo build --release --manifest-path $(CARGO_TOML)

install:
	@echo "Installing $(BINARY_NAME)..."
	# Create user/group if not exists
	id -u $(USER) &>/dev/null || useradd -r -s /usr/sbin/nologin $(USER)
	
	# Install binary
	install -m 755 $(TARGET_RELEASE) $(INSTALL_BIN_DIR)/$(BINARY_NAME)
	
	# Install config
	mkdir -p $(INSTALL_CONF_DIR)
	[ -f $(INSTALL_CONF_DIR)/rsudp.toml ] || install -m 640 -o root -g $(GROUP) rsudp-rust/rsudp_settings.toml $(INSTALL_CONF_DIR)/rsudp.toml
	
	# Create data directory
	mkdir -p $(INSTALL_DATA_DIR)
	chown $(USER):$(GROUP) $(INSTALL_DATA_DIR)
	chmod 750 $(INSTALL_DATA_DIR)
	
	# Install systemd service
	install -m 644 $(SERVICE_FILE) $(SYSTEMD_DIR)/$(SERVICE_NAME).service
	
	# Reload systemd
	systemctl daemon-reload
	
	@echo "Installation complete. Run 'systemctl start $(SERVICE_NAME)' to start the service."

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