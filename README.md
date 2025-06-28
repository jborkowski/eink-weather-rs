# ESP E-Ink Weather Station

### ESP Rust Toolchain Setup

This project uses the ESP Rust toolchain. To set up the required toolchain:

1. Install `espup` if you haven't already:
   ```bash
   cargo install espup
   ```

2. Install the ESP toolchain version 1.82.0:
   ```bash
   espup install --toolchain-version=1.72.0 --name esp-1.82
   ```

3. Load the ESP environment variables in your current shell:
   ```bash
   . $HOME/export-esp.sh
   ```
