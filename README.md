![StarDelta Logo](assets/StarDelta%20Logo.svg)


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

### Platform Support

Official releases are built for Windows only. However, as a Tauri-based app, you can build custom releases for MacOS and Linux.

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

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
