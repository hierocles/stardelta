# ![StarDelta Logo](assets/StarDelta%20Logo.svg)

StarDelta is a tool for creating and applying patches to Starfield UIs. It includes special support for modifying SWF files.

## Features

- Convert SWF files to JSON for editing
- Edit SWF files via JSON modification files
- Avoid the need to directly edit and redistribute closed-source SWF files
- Apply patches to single files or batch process multiple files
- Includes a general xdelta3 patcher for patching any binary file

## Usage

### Binary Patching

1. Select the original file to patch
2. Select the delta file to apply
3. Select the output directory
4. Click "Patch"

### SWF Patching

You can patch SWF files either individually or in batch.

#### Individual Patching

1. Select the original SWF file
2. Select the JSON patch file
3. Select the output directory (typically "Interface")
4. Click "Patch"

#### Batch Patching

1. Select the JSON configuration file
2. Select the original SWF files to patch
3. Select the output directory (typically "Interface")
4. Click "Patch"

#### Installing Patched Files

1. Move the patched files to the Starfield Data directory, overwriting if necessary
2. Alternatively, use your preferred package manager to install the "Interface" folder as a mod

## JSON Patch Format

The JSON patch format is a simple way to describe changes to SWF files. It is a list of operations to perform on the SWF file.

The patch file must include the `swf` section, while `transparent` and `file` operations are optional:

```json
{
  "transparent": [],  // Optional: Can be omitted if not making shapes transparent
  "file": [],        // Optional: Can be omitted if not replacing shapes with SVG files
  "swf": {           // Required: Must be present even if modifications array is empty
    "modifications": []
  }
}
```

### Transparent

The transparent operation is optional and used to make specific shapes transparent. If you don't need to make any shapes transparent, you can omit this section entirely.

```json
{
  "transparent": [138, 139, 140],  // Array of shape IDs to make transparent
  "swf": {
    "modifications": []
  }
}
```

### File

The file operation is optional and used to replace specific shapes with new shapes from SVG files. If you don't need to replace any shapes, you can omit this section entirely. When used, the source path should be relative to the JSON patch file's location.

Note: Only SVG files are supported for the source. SVG files should be placed in the same directory as the patch file or in a subdirectory.

```json
{
  "file": [
    {
      "source": "new_shapes.svg",  // Path relative to the JSON patch file
      "shapes": [1, 3, 6, 8]       // Shape IDs to replace
    }
  ],
  "swf": {
    "modifications": []
  }
}
```

### SWF

The swf operation is used to modify the SWF file's attributes. Supported tags are defined in the [open-flash/swf-types](https://github.com/open-flash/swf-types) repository.

SWF tags must conform to the [SWF 19.0 specification](https://open-flash.github.io/mirrors/swf-spec-19.pdf). Any tags that are not supported will cause an error or unexpected behavior. Tag properties must also conform to the specification.

Each tag modification must include:

- `tag`: The tag type name (e.g., "DefineShapeTag", "DefineEditTextTag")
- `id`: The unique identifier for the tag (except for some tags like FileAttributesTag)
- `properties`: Object containing the properties to modify, which vary by tag type

#### Common Tag Types

Here are some commonly used tag types and their properties:

**DefineEditTextTag** - Modifies text fields

```json
{
  "tag": "DefineEditTextTag",
  "id": 5,
  "properties": {
    "bounds": {
      "x_min": 0,
      "x_max": 100,
      "y_min": 0,
      "y_max": 20
    },
    "font_id": 3,
    "font_class": "Arial",
    "font_size": 12,
    "color": {
      "type": "RGB",
      "red": 16,
      "green": 22,
      "blue": 32
    },
    "text": "Hello World"
  }
}
```

**DefineShapeTag** - Modifies shapes

```json
{
  "tag": "DefineShapeTag",
  "id": 10,
  "properties": {
    "bounds": {
      "x_min": 0,
      "x_max": 100,
      "y_min": 0,
      "y_max": 100
    },
    "styles": {
      "fill": [
        {
          "type": "solid",
          "color": {
            "type": "RGB",
            "red": 255,
            "green": 0,
            "blue": 0
          }
        }
      ],
      "line": []
    }
  }
}
```

**FileAttributesTag** - Sets global SWF attributes

```json
{
  "tag": "FileAttributesTag",
  "properties": {
    "useNetwork": false,
    "useGPU": false,
    "hasMetadata": false,
    "actionScript3": true
  }
}
```

### Batch Configuration

The batch configuration file (`configuration.json`) is a JSON file that describes the patches to apply to the SWF files. Use appropriate names for the mods, so that end users can easily identify which SWF file they need to choose.

This file is not required for individual patching. However, if you are patching multiple SWF files, you are encouraged to provide it.

The `ba2` field is optional and used to patch a BA2 archive. If provided, the `files` field is required and must be an array of objects with `path` and `config` fields. The `path` field is the path to the SWF file to patch (within the BA2 archive), and the `config` field is the path to the patch file (relative to the configuration file).

```json
{
  "mods": [
    {
      "ba2": true,
      "name": "Starfield - Interface.ba2",
      "files": [
        {
            "path": "interface/datamenu.swf",
            "config": "patches/data-menu.json"
        },
        {
            "path": "interface/shipcrewmenu.swf",
            "config": "patches/shipcrew-menu.json"
        },

        {
            "path": "interface/skillsmenu.swf",
            "config": "patches/skills-menu.json"
        }
      ]
    },
    {
      "name": "Barter Menu (bartermenu.swf)",
      "config": "patches/barter-menu.json"
    },

    {
      "name": "Container Menu (containermenu.swf)",
      "config": "patches/container-menu.json"
    },
    {
      "name": "Inventory Menu (inventorymenu.swf)",
      "config": "patches/inventory-menu.json"
    }
  ]
}
```

## Packaging Patch Mods

The recommended folder structure for patch mods is as follows:

```sh
Root/
  Interface/                  # Output directory for patched SWF files
  StarDelta/
    ModName/
      patches/               # Directory containing patch JSON files
        patch.json
        patch2.json
        ...
      assets/               # Directory containing SVG files
        shape1.svg
        shape2.svg
        ...
      configuration.json    # Batch configuration file
```

Users should be instructed to choose the Interface folder as the output directory when patching.

## Building

### Build Prerequisites

The following tools are required to build StarDelta:

- [Rust](https://www.rust-lang.org/tools/install) - Latest stable version
- [Node.js](https://nodejs.org/) - Version 16 or later
- [Tauri CLI](https://v2.tauri.app/reference/cli/i)

Additional platform-specific requirements:

- Windows: Microsoft Visual Studio C++ Build Tools
- Linux: Development packages (see Tauri prerequisites)
- macOS: Xcode Command Line Tools

### Build Instructions

1. Clone the repository:

   ```sh
   git clone https://github.com/hierocles/stardelta.git
   cd stardelta
   ```

2. Install frontend dependencies:

   ```sh
   npm install
   ```

3. Build the Tauri application:

   ```sh
   npm run tauri build
   ```

4. The built application will be available in the `src-tauri/target/release` directory.

### Platform Support

StarDelta is primarily developed and tested on Windows, as it targets Starfield UI modifications. While the core functionality should work on other platforms, some features may be limited:

- **Windows**: Fully supported
- **Linux**: Basic functionality works, but not officially supported
- **macOS**: Basic functionality works, but not officially supported

## Troubleshooting

### Common Issues

1. **SVG Import Fails**
   - Ensure SVG files use absolute coordinates
   - Check that all paths are properly closed
   - Verify the SVG file is in the correct directory relative to the JSON patch

2. **Shape Replacement Issues**
   - Verify shape IDs match the ones in the original SWF
   - Check that SVG dimensions are appropriate for the target shape
   - Ensure all required styles are specified

3. **Batch Processing Errors**
   - Verify all paths in configuration.json are correct
   - Check that all referenced JSON patch files exist
   - Ensure output directory is writable

### Getting Help

If you encounter issues not covered here:

1. Check the [GitHub Issues](https://github.com/hierocles/stardelta/issues) for similar problems
2. Enable debug logging by setting the environment variable `RUST_LOG=debug`
3. Open a new issue with:
   - The error message
   - The JSON patch file content
   - The debug logs
   - Steps to reproduce the issue
   - Link to the original SWF file

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

StarDelta is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

### Third-Party Licenses

StarDelta uses several open-source components, each with their own licenses:

#### Core Dependencies

- **[open-flash/swf-types](https://github.com/open-flash/swf-types)** - MIT License
  - Used for SWF file format definitions and handling
- **[swf-parser](https://github.com/open-flash/swf-parser)** - MIT License
  - Used for parsing SWF files
- **[swf-emitter](https://github.com/open-flash/swf-emitter)** - MIT License
  - Used for writing SWF files

#### Application Framework

- **[Tauri](https://tauri.app)** - MIT or Apache 2.0 License
  - Used as the application framework
  - Some Tauri components may include additional dependencies with compatible open-source licenses

#### SVG Processing

- **[kurbo](https://github.com/linebender/kurbo)** - Apache 2.0 License
  - Used for path geometry calculations
- **[svgtypes](https://github.com/RazrFalcon/svgtypes)** - MIT License
  - Used for SVG parsing

#### Binary Diff Tools

- **[xdelta3](http://xdelta.org/)** - Apache 2.0 License
- **[xdelta3-rs](https://github.com/liushuyu/xdelta3-rs)**
  - Used for binary patch creation and application

#### Frontend Dependencies

The frontend uses various NPM packages, each with their own licenses. Key dependencies include:

- React - MIT License
- TypeScript - Apache 2.0 License

### Legal Notes

1. **Starfield Assets**: This tool does not distribute any Starfield game assets. Users are responsible for ensuring they have the necessary rights to modify game files.

2. **Modified SWF Files**: When distributing mods created with StarDelta, ensure you:
   - Do not include original game assets
   - Only distribute the patch files
   - Include appropriate attribution and licenses
   - Follow Bethesda's modding guidelines

3. **Contributions**: By contributing to StarDelta, you agree that your contributions will be licensed under the same MIT License as the project.

For a complete list of dependencies and their licenses, see:

- `Cargo.toml` for Rust dependencies
- `package.json` for Node.js dependencies
