"use client"

import { useState, useEffect, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"
import { open, save } from "@tauri-apps/plugin-dialog"
import { Window, PhysicalSize } from "@tauri-apps/api/window"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { NavLink } from "react-router"
import { Logo } from "@/components/ui/logo"
import { toast } from "sonner"

const MAX_FILE_SIZE = 100 * 1024 * 1024 // 100MB

interface ModConfig {
  ba2?: boolean;
  name: string;
  config?: string;
  files?: Array<{
    path: string;
    config: string;
  }>;
}

interface SwfMapping {
  mod_name: string;
  swf_path: string;
}

export function SwfPatcher() {
  // End-user states
  const [originalSwfPath, setOriginalSwfPath] = useState("")
  const [modJsonPath, setModJsonPath] = useState("")
  const [outputPath, setOutputPath] = useState("")
  const [isApplyingMod, setIsApplyingMod] = useState(false)
  const [isExportingJson, setIsExportingJson] = useState(false)

  // Batch processing states
  const [batchConfigPath, setBatchConfigPath] = useState("")
  const [batchOutputDir, setBatchOutputDir] = useState("")
  const [isBatchProcessing, setIsBatchProcessing] = useState(false)
  const [modConfigs, setModConfigs] = useState<ModConfig[]>([])
  const [selectedBa2Path, setSelectedBa2Path] = useState<string>("")
  const [swfMappings, setSwfMappings] = useState<SwfMapping[]>([])

  // Mod author states
  const [sourceSwfPath, setSourceSwfPath] = useState("")
  const [exportJsonPath, setExportJsonPath] = useState("")

  // Add current tab state
  const [currentTab, setCurrentTab] = useState("user")

  const contentRef = useRef<HTMLDivElement>(null)
  const resizeTimeout = useRef<NodeJS.Timeout | undefined>(undefined)

  // Remove all resize effects and just do initial window setup
  useEffect(() => {
    const setupWindow = async () => {
      try {
        const window = Window.getCurrent();
        await window.setSize(new PhysicalSize(1200, 1000));
        await window.setMinSize(new PhysicalSize(1200, 1000));
        await window.setMaxSize(new PhysicalSize(1200, 1000));
      } catch (err) {
        console.error("Failed to configure window:", err);
      }
    };
    setupWindow();
  }, []);

  const handleSelectFile = async (
    setter: (path: string) => void,
    filters: { name: string; extensions: string[] }[]
  ) => {
    try {
      const selected = await open({
        multiple: false,
        filters,
      })
      if (selected && typeof selected === "string") {
        // Check file extension
        const ext = selected.split('.').pop()?.toLowerCase()
        if (filters.length > 0 && !filters[0].extensions.includes(ext || '')) {
          toast.error(`Invalid file type. Expected ${filters[0].extensions.map(e => `.${e}`).join(" or ")} file`)
          return
        }

        // Check file size
        try {
          const size = await invoke<number>("get_file_size", { path: selected })
          if (size > MAX_FILE_SIZE) {
            toast.warning(`Large file detected (${Math.round(size / 1024 / 1024)}MB). Processing may take longer.`, {
              duration: 6000,
            })
          }
        } catch (err) {
          console.error("Failed to check file size:", err)
        }

        setter(selected)
      }
    } catch (err) {
      toast.error("Failed to select file")
    }
  }

  const handleSaveFile = async (
    setter: (path: string) => void,
    filters: { name: string; extensions: string[] }[]
  ) => {
    try {
      const selected = await save({
        filters,
      })
      if (selected) {
        setter(selected)
      }
    } catch (err) {
      toast.error("Failed to select save location")
    }
  }

  const handleApplyMod = async () => {
    try {
      if (!originalSwfPath) throw new Error("Please select the original SWF file")
      if (!modJsonPath) throw new Error("Please select the mod's JSON file")
      if (!outputPath) throw new Error("Please specify where to save the modified SWF")

      setIsApplyingMod(true)
      toast.loading("Applying mod...", { id: "apply-mod" })

      // First convert original SWF to temporary JSON
      const tempJsonPath = originalSwfPath + ".temp.json"
      await invoke("convert_swf_to_json", {
        swfPath: originalSwfPath,
        jsonPath: tempJsonPath
      })

      // Apply the mod's modifications
      await invoke("apply_json_modifications", {
        swfJsonPath: tempJsonPath,
        configJsonPath: modJsonPath,
        outputJsonPath: tempJsonPath
      })

      // Convert back to SWF
      await invoke("convert_json_to_swf", {
        jsonPath: tempJsonPath,
        swfPath: outputPath
      })

      toast.success("Mod applied successfully!", { id: "apply-mod" })
    } catch (err) {
      console.error("Error applying mod:", err)
      toast.error(err instanceof Error ? err.message : "Failed to apply mod", { id: "apply-mod" })
    } finally {
      setIsApplyingMod(false)
    }
  }

  const handleExportSwfJson = async () => {
    try {
      if (!sourceSwfPath) throw new Error("Please select the SWF file to analyze")
      if (!exportJsonPath) throw new Error("Please specify where to save the JSON")

      setIsExportingJson(true)
      toast.loading("Exporting SWF to JSON...", { id: "export-json" })

      await invoke("convert_swf_to_json", {
        swfPath: sourceSwfPath,
        jsonPath: exportJsonPath
      })

      toast.success("SWF exported to JSON successfully! You can now create your modification JSON file.", { id: "export-json" })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to export SWF to JSON", { id: "export-json" })
    } finally {
      setIsExportingJson(false)
    }
  }

  const handleSelectDirectory = async (
    setter: (path: string) => void
  ) => {
    try {
      const selected = await open({
        directory: true,
      })
      if (selected && typeof selected === "string") {
        setter(selected)
      }
    } catch (err) {
      toast.error("Failed to select directory")
    }
  }

  const handleLoadBatchConfig = async (path: string) => {
    try {
      const configJson = await invoke("read_file_to_string", { path })
      const config = JSON.parse(configJson as string)
      setModConfigs(config.mods || [])

      // Clear previous mappings when loading new config
      setSwfMappings([])
      setSelectedBa2Path("")
    } catch (error) {
      console.error("Failed to load batch config:", error)
      toast.error("Failed to load batch configuration")
    }
  }

  const handleSelectBatchConfig = async () => {
    const selected = await open({
      filters: [{ name: "JSON", extensions: ["json"] }],
    })
    if (selected) {
      setBatchConfigPath(selected)
      await handleLoadBatchConfig(selected)
    }
  }

  const handleSelectBa2 = async () => {
    const selected = await open({
      filters: [{ name: "BA2 Archive", extensions: ["ba2"] }],
    })
    if (selected) {
      setSelectedBa2Path(selected)
    }
  }

  const handleSetSwfPath = async (modName: string) => {
    const selected = await open({
      filters: [{ name: "SWF", extensions: ["swf"] }],
    })
    if (selected) {
      setSwfMappings(prev => [
        ...prev.filter(m => m.mod_name !== modName),
        { mod_name: modName, swf_path: selected }
      ])
    }
  }

  const handleBatchProcess = async () => {
    if (!batchConfigPath || !batchOutputDir) {
      toast.error("Please select a configuration file and output directory")
      return
    }

    // For BA2 mods, check if BA2 file is selected
    const hasBa2Mods = modConfigs.some(mod => mod.ba2)
    if (hasBa2Mods && !selectedBa2Path) {
      toast.error("Please select the BA2 archive")
      return
    }

    // For non-BA2 mods, check if all SWFs are mapped
    const nonBa2Mods = modConfigs.filter(mod => !mod.ba2)
    const allMapped = nonBa2Mods.every(mod =>
      swfMappings.some(mapping => mapping.mod_name === mod.name)
    )
    if (nonBa2Mods.length > 0 && !allMapped) {
      toast.error("Please select SWF files for all non-BA2 mods")
      return
    }

    try {
      setIsBatchProcessing(true)

      const result = await invoke("batch_process_swf", {
        config: {
          config_file: batchConfigPath,
          output_directory: batchOutputDir,
          ba2_path: selectedBa2Path || undefined,
        }
      })

      toast.success("Batch processing completed successfully")
      console.log("Processed files:", result)
    } catch (error) {
      console.error("Batch processing failed:", error)
      toast.error("Batch processing failed")
    } finally {
      setIsBatchProcessing(false)
    }
  }

  return (
    <div className="flex flex-col h-screen">
      <header className="sticky top-5 z-50">
        <div className="w-[600px] mx-auto relative px-4">
          <NavLink
            to="/"
            className="fixed top-4 left-4 inline-flex items-center text-sm text-muted-foreground hover:text-foreground z-50 p-4 -m-4"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="mr-2 h-4 w-4"
            >
              <path d="m12 19-7-7 7-7"/>
              <path d="M19 12H5"/>
            </svg>
            Back to Tools
          </NavLink>

          <div className="py-8">
            <Logo />
          </div>
        </div>
      </header>

      <main className="flex-1 overflow-auto">
        <div className="w-[600px] mx-auto px-4 py-8">
          <Tabs defaultValue="batch" onValueChange={setCurrentTab}>
            <TabsList className="grid grid-cols-3 mb-4">
              <TabsTrigger value="user">Patch Single File</TabsTrigger>
              <TabsTrigger value="batch">Patch Multiple Files</TabsTrigger>
              <TabsTrigger value="author">Mod Author Tools</TabsTrigger>
            </TabsList>

            <TabsContent value="user" className="space-y-4">
              <Card className="min-h-[400px]">
                <CardHeader>
                  <CardTitle>Install SWF Patch</CardTitle>
                  <CardDescription>
                    Apply a patch to your SWF file using the modification files provided by the mod author
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-2">
                    <Label htmlFor="original-swf">Original SWF File</Label>
                    <div className="flex gap-2">
                      <Input
                        id="original-swf"
                        value={originalSwfPath}
                        readOnly
                        placeholder="Select your original SWF file..."
                      />
                      <Button
                        onClick={() =>
                          handleSelectFile(setOriginalSwfPath, [
                            { name: "SWF Files", extensions: ["swf"] },
                          ])
                        }
                      >
                        Browse
                      </Button>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="mod-json">Patch Files</Label>
                    <div className="flex gap-2">
                      <Input
                        id="mod-json"
                        value={modJsonPath}
                        readOnly
                        placeholder="Select the patch's JSON file..."
                      />
                      <Button
                        onClick={() =>
                          handleSelectFile(setModJsonPath, [
                            { name: "JSON Files", extensions: ["json"] },
                          ])
                        }
                      >
                        Browse
                      </Button>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      Select the JSON file provided by the mod author. Make sure all SVG files are in the same directory.
                    </p>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="output-swf">Save Modified SWF As</Label>
                    <div className="flex gap-2">
                      <Input
                        id="output-swf"
                        value={outputPath}
                        readOnly
                        placeholder="Choose where to save the modified SWF..."
                      />
                      <Button
                        onClick={() =>
                          handleSaveFile(setOutputPath, [
                            { name: "SWF Files", extensions: ["swf"] },
                          ])
                        }
                      >
                        Browse
                      </Button>
                    </div>
                  </div>

                  <Button
                    className="w-full"
                    onClick={handleApplyMod}
                    disabled={!originalSwfPath || !modJsonPath || !outputPath || isApplyingMod}
                  >
                    {isApplyingMod ? "Patching..." : "Patch File"}
                  </Button>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="batch" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Batch Process SWF Files</CardTitle>
                  <CardDescription>
                    Process multiple SWF files using a batch configuration file.
                    Supports both loose SWF files and files within BA2 archives.
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-2">
                    <Label>Configuration File</Label>
                    <div className="flex gap-2">
                      <Input
                        value={batchConfigPath}
                        readOnly
                        placeholder="Select batch configuration file..."
                      />
                      <Button onClick={handleSelectBatchConfig} disabled={isBatchProcessing}>
                        Browse
                      </Button>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Label>Output Directory</Label>
                    <div className="flex gap-2">
                      <Input
                        value={batchOutputDir}
                        readOnly
                        placeholder="Select where to save processed files..."
                      />
                      <Button
                        onClick={() => handleSelectDirectory(setBatchOutputDir)}
                        disabled={isBatchProcessing}
                      >
                        Browse
                      </Button>
                    </div>
                  </div>

                  {modConfigs.some(mod => mod.ba2) && (
                    <div className="space-y-2">
                      <Label>BA2 Archive</Label>
                      <div className="flex gap-2">
                        <Input
                          value={selectedBa2Path}
                          readOnly
                          placeholder="Select BA2 archive..."
                        />
                        <Button
                          onClick={handleSelectBa2}
                          disabled={isBatchProcessing}
                        >
                          Browse
                        </Button>
                      </div>
                      <p className="text-sm text-muted-foreground">
                        Select the BA2 archive containing the SWF files to patch
                      </p>
                    </div>
                  )}

                  {modConfigs.filter(mod => !mod.ba2).map((mod, index) => (
                    <div key={index} className="space-y-2">
                      <Label>{mod.name}</Label>
                      <div className="flex gap-2">
                        <Input
                          value={swfMappings.find(m => m.mod_name === mod.name)?.swf_path || ""}
                          readOnly
                          placeholder="Select SWF file..."
                        />
                        <Button
                          onClick={() => handleSetSwfPath(mod.name)}
                          disabled={isBatchProcessing}
                        >
                          Browse
                        </Button>
                      </div>
                    </div>
                  ))}

                  <Button
                    className="w-full"
                    onClick={handleBatchProcess}
                    disabled={isBatchProcessing}
                  >
                    {isBatchProcessing ? "Patching..." : "Patch Files"}
                  </Button>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="author" className="space-y-4">
              <Card className="min-h-[400px]">
                <CardHeader>
                  <CardTitle>Create SWF Mod</CardTitle>
                  <CardDescription>
                    Tools for mod authors to analyze SWF files and create modifications
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-6">
                  <div className="rounded-lg border p-4">
                    <h3 className="font-medium mb-2">Export SWF to JSON</h3>
                    <p className="text-sm text-muted-foreground mb-4">
                      Export the SWF file to JSON format for analysis. Use this to understand the structure
                      and create your modification JSON file.
                    </p>
                    <div className="space-y-4">
                      <div className="flex gap-2">
                        <Input
                          value={sourceSwfPath}
                          readOnly
                          placeholder="Select SWF file to analyze..."
                        />
                        <Button
                          onClick={() =>
                            handleSelectFile(setSourceSwfPath, [
                              { name: "SWF Files", extensions: ["swf"] },
                            ])
                          }
                        >
                          Browse
                        </Button>
                      </div>
                      <div className="flex gap-2">
                        <Input
                          value={exportJsonPath}
                          readOnly
                          placeholder="Save analysis JSON as..."
                        />
                        <Button
                          onClick={() =>
                            handleSaveFile(setExportJsonPath, [
                              { name: "JSON Files", extensions: ["json"] },
                            ])
                          }
                        >
                          Browse
                        </Button>
                      </div>
                      <div className="flex flex-col gap-2">
                        <Button
                          onClick={handleExportSwfJson}
                          disabled={!sourceSwfPath || !exportJsonPath || isExportingJson}
                        >
                          {isExportingJson ? "Exporting..." : "Export to JSON"}
                        </Button>
                        <p className="text-xs text-muted-foreground text-center">
                          After exporting, create your modification JSON file and place your SVG files in the same directory.
                        </p>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </div>
      </main>
    </div>
  )
}
