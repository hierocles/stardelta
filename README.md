# StarDelta

StarDelta is an xdelta3 patcher for Starfield UIs. It allows you to create and apply patches to files using the xdelta3 algorithm.

## Features

- Create patches from original and edited files.
- Apply patches to files.
- Save patches and patched files to user-specified directories.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (for building the frontend)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites#installing-tauri-cli)

## Installation

1. Clone the repository:

   ```sh
   git clone https://github.com/yourusername/stardelta.git
   cd stardelta
   ```

2. Install dependencies:

   ```sh
   npm install
   ```

## Building

1. Build the Tauri application:

   ```sh
   npm run tauri build
   ```

2. The built application will be available in the `src-tauri/target/release` directory.

## Running in Development

1. Start the Tauri development server:

   ```sh
   npm run tauri dev
   ```

2. The application will open in a new window.

## Usage

1. Open the application.

2. To create a patch:
   - Select the "Create Patch" tab.
   - Click "Browse" to select the original file.
   - Click "Browse" to select the edited file.
   - Click "Browse" to select the output directory.
   - Click "Create Patch" to create the patch.

3. To apply a patch:
   - Select the "Apply Patch" tab.
   - Click "Browse" to select the file to patch.
   - Click "Browse" to select the patch file.
   - Click "Browse" to select the output directory.
   - Click "Apply Patch" to apply the patch.

## Logging

The application uses the `tauri_plugin_log` plugin for logging and error tracing. Logs are available in the webview console.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
