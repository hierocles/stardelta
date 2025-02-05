# ![StarDelta Logo](assets/StarDelta%20Logo.svg)

StarDelta is an xdelta3 patcher for Starfield UIs. It allows you to create and apply patches to files using the xdelta3 algorithm, and includes special support for modifying SWF files.

## Features

- Create patches from original and edited files
- Apply patches to files
- Save patches and patched files to user-specified directories
- Special SWF file support:
  - Convert SWF files to JSON for editing
  - Replace shapes in SWF files with SVG shapes
  - Modify SWF properties like background color and bounds
  - Convert modified JSON back to SWF

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

### Basic Patching

1. To create a patch:
   - Select the "Create Patch" tab
   - Click "Browse" to select the original file
   - Click "Browse" to select the edited file
   - Click "Browse" to select the output directory
   - Click "Create Patch" to create the patch

2. To apply a patch:
   - Select the "Apply Patch" tab
   - Click "Browse" to select the file to patch
   - Click "Browse" to select the patch file
   - Click "Browse" to select the output directory
   - Click "Apply Patch" to apply the patch

### SWF Modification

1. Convert SWF to JSON:

   ```sh
   # Using the UI:
   1. Select the SWF file
   2. Choose "Convert to JSON"
   3. Select output location
   ```

2. Replace shapes with SVG:
   - Create a configuration JSON file:

   ```json
   {
     "file": [
       {
         "source": "path/to/shapes.svg",
         "shapes": [1, 3, 6]  // Shape IDs to replace
       }
     ],
     "swf": {
       "bounds": {
         "x": { "min": 0, "max": 960 },
         "y": { "min": 0, "max": 540 }
       },
       "modifications": [
         {
           "tag": "SetBackgroundColorTag",
           "id": 0,
           "properties": {
             "backgroundColor": {
               "r": 255,
               "g": 255,
               "b": 255,
               "a": 255
             }
           }
         }
       ]
     }
   }
   ```

   - Apply modifications:

   ```sh
   # Using the UI:
   1. Select the JSON file
   2. Choose "Apply Modifications"
   3. Select the configuration file
   4. Choose output location
   ```

3. Convert back to SWF:

   ```sh
   # Using the UI:
   1. Select the modified JSON file
   2. Choose "Convert to SWF"
   3. Select output location
   ```

### SVG Support

The tool supports converting SVG shapes to SWF format with the following features:

- Path commands: MoveTo, LineTo, CurveTo (cubic beziers approximated as quadratic)
- Fill and stroke styles with opacity
- Transform attributes (translate, scale, rotate, matrix)
- Automatic bounds calculation with padding

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
