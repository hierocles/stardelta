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
  name: string;
  config: string;
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
  const [swfMappings, setSwfMappings] = useState<Array<{ mod_name: string, swf_path: string }>>([])

  // Mod author states
  const [sourceSwfPath, setSourceSwfPath] = useState("")
  const [exportJsonPath, setExportJsonPath] = useState("")

  // Add current tab state
  const [currentTab, setCurrentTab] = useState("user")

  const contentRef = useRef<HTMLDivElement>(null)
  const resizeTimeout = useRef<NodeJS.Timeout | undefined>(undefined)

  // Add effect to handle form input changes
  useEffect(() => {
    if (!contentRef.current) return;
    const rect = contentRef.current.getBoundingClientRect();
    const window = Window.getCurrent();
    window.setSize(new PhysicalSize(600, Math.max(600, rect.height + 180)));
  }, [originalSwfPath, modJsonPath, outputPath, batchConfigPath, batchOutputDir, sourceSwfPath, exportJsonPath, modConfigs.length]);


  // Initial window setup
  useEffect(() => {
    const setupWindow = async () => {
      try {
        const window = Window.getCurrent();
        await window.setMinSize(new PhysicalSize(600, 600));
        await window.setContentProtected(false);
        await window.setSize(new PhysicalSize(600, 600));
      } catch (err) {
        console.error("Failed to configure window:", err);
      }
    };
    setupWindow();
  }, []);

  // Smooth resize effect for content changes
  useEffect(() => {
    if (!contentRef.current) return;

    const updateWindowSize = async (rect: DOMRect) => {
      try {
        const window = Window.getCurrent();
        const newHeight = Math.max(600, rect.height + 180);
        await window.setSize(new PhysicalSize(600, newHeight));


      } catch (err) {
        console.error("Failed to update window size:", err);
      }
    };

    const resizeObserver = new ResizeObserver((entries) => {
      if (resizeTimeout.current) {
        clearTimeout(resizeTimeout.current);
      }
      resizeTimeout.current = setTimeout(() => {
        updateWindowSize(entries[0].target.getBoundingClientRect());
      }, 100);
    });

    resizeObserver.observe(contentRef.current);
    return () => {
      resizeObserver.disconnect();
      if (resizeTimeout.current) {
        clearTimeout(resizeTimeout.current);
      }
    };
  }, []);

  // Handle tab changes
  useEffect(() => {
    // Wait for the tab content to render
    const timeout = setTimeout(async () => {
      if (contentRef.current) {
        const rect = contentRef.current.getBoundingClientRect();
        const window = await Window.getCurrent();
        await window.setSize(new PhysicalSize(600, Math.max(600, rect.height + 180)));
      }
    }, 100); // Increased timeout to match other resize operations


    return () => clearTimeout(timeout);
  }, [currentTab]);

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
      const configJson = await invoke<string>("read_file_to_string", { path })
      const config = JSON.parse(configJson)
      setModConfigs(config.mods)
      setSwfMappings(config.mods.map((mod: ModConfig) => ({ mod_name: mod.name, swf_path: "" })))
    } catch (err) {
      console.error("Error loading batch config:", err)
      toast.error("Failed to load configuration file")
    }
  }

  const handleSelectBatchConfig = async () => {
    await handleSelectFile(async (path) => {
      setBatchConfigPath(path)
      await handleLoadBatchConfig(path)
    }, [{ name: "JSON Files", extensions: ["json"] }])
  }

  const handleSetSwfPath = async (modName: string) => {
    await handleSelectFile((path) => {
      setSwfMappings(prev => prev.map(mapping =>
        mapping.mod_name === modName ? { ...mapping, swf_path: path } : mapping
      ))
    }, [{ name: "SWF Files", extensions: ["swf"] }])
  }

  const handleBatchProcess = async () => {
    try {
      if (!batchConfigPath) throw new Error("Please select the batch configuration file")
      if (!batchOutputDir) throw new Error("Please specify where to save the processed files")
      if (swfMappings.some(m => !m.swf_path)) throw new Error("Please select SWF files for all mods")

      setIsBatchProcessing(true)
      toast.loading("Processing SWF files...", { id: "batch-process" })

      const processedFiles = await invoke<string[]>("batch_process_swf", {
        config: {
          config_file: batchConfigPath,
          output_directory: batchOutputDir,
          swf_mappings: swfMappings,
        }
      })

      toast.success(`Successfully processed ${processedFiles.length} files!`, { id: "batch-process" })
    } catch (err) {
      console.error("Error in batch processing:", err)
      toast.error(err instanceof Error ? err.message : "Failed to process files", { id: "batch-process" })
    } finally {
      setIsBatchProcessing(false)
    }
  }

  return (
    <div className="flex flex-col min-h-screen overflow-hidden">
      <header className="sticky top-10 z-50">
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

      <main className="flex-1 py-8">
        <div className="w-[600px] mx-auto px-4 overflow-hidden" ref={contentRef}>
          <Tabs defaultValue="batch" onValueChange={setCurrentTab}>
            <TabsList className="grid grid-cols-3 mb-4">
              <TabsTrigger value="user">Install Mod</TabsTrigger>
              <TabsTrigger value="batch">Batch Process</TabsTrigger>
              <TabsTrigger value="author">Create Mod</TabsTrigger>
            </TabsList>

            <TabsContent value="user">
              <Card className="min-h-[400px]">
                <CardHeader>
                  <CardTitle>Install SWF Mod</CardTitle>

                  <CardDescription>
                    Apply a mod to your SWF file using the modification files provided by the mod author
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
                    <Label htmlFor="mod-json">Mod Files</Label>
                    <div className="flex gap-2">
                      <Input
                        id="mod-json"
                        value={modJsonPath}
                        readOnly
                        placeholder="Select the mod's JSON file..."
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
                    {isApplyingMod ? "Applying Mod..." : "Apply Mod"}
                  </Button>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="batch">
              <Card className="min-h-[400px]">
                <CardHeader>
                  <CardTitle>Batch Process SWF Files</CardTitle>
                  <CardDescription>
                    Process multiple SWF files using a batch configuration file
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-2">
                    <Label htmlFor="batch-config">Batch Configuration</Label>
                    <div className="flex gap-2">
                      <Input
                        id="batch-config"
                        value={batchConfigPath}
                        readOnly
                        placeholder="Select the batch configuration file..."
                      />
                      <Button
                        onClick={handleSelectBatchConfig}
                        disabled={isBatchProcessing}
                      >
                        Browse
                      </Button>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      Select the JSON configuration file that contains the mod configurations.
                    </p>
                  </div>

                  {modConfigs.length > 0 && (
                    <div className="space-y-2">
                      <Label>SWF File Mappings</Label>
                      <div className="space-y-2">
                        {swfMappings.map((mapping, index) => (
                          <div key={index} className="flex items-start gap-2 p-2 border rounded-lg">
                            <div className="flex-1 space-y-1">
                              <p className="text-sm font-medium">
                                {mapping.mod_name}
                              </p>
                              <div className="flex gap-2">
                                <Input
                                  value={mapping.swf_path ? mapping.swf_path.split('\\').pop() : ''}
                                  readOnly
                                  placeholder="Select SWF file..."
                                  className="flex-1"
                                />
                                <Button
                                  onClick={() => handleSetSwfPath(mapping.mod_name)}
                                  disabled={isBatchProcessing}
                                >
                                  Browse
                                </Button>
                              </div>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  <div className="space-y-2">
                    <Label htmlFor="batch-output">Output Directory</Label>
                    <div className="flex gap-2">
                      <Input
                        id="batch-output"
                        value={batchOutputDir}
                        readOnly
                        placeholder="Choose where to save processed files..."
                      />
                      <Button
                        onClick={() => handleSelectDirectory(setBatchOutputDir)}
                        disabled={isBatchProcessing}
                      >
                        Browse
                      </Button>
                    </div>
                  </div>

                  <Button
                    className="w-full"
                    onClick={handleBatchProcess}
                    disabled={!batchConfigPath || !batchOutputDir || swfMappings.some(m => !m.swf_path) || isBatchProcessing}
                  >
                    {isBatchProcessing ? "Processing Files..." : "Process Files"}
                  </Button>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="author">
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
