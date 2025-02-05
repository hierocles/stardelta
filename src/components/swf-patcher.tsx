"use client"

import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { open, save } from "@tauri-apps/plugin-dialog"
import { stat } from "@tauri-apps/plugin-fs"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { NavLink } from "react-router"
import { Logo } from "@/components/ui/logo"
import { toast } from "sonner"

const MAX_FILE_SIZE = 100 * 1024 * 1024 // 100MB

export function SwfPatcher() {
  // End-user states
  const [originalSwfPath, setOriginalSwfPath] = useState("")
  const [modJsonPath, setModJsonPath] = useState("")
  const [outputPath, setOutputPath] = useState("")
  const [isApplyingMod, setIsApplyingMod] = useState(false)
  const [isExportingJson, setIsExportingJson] = useState(false)

  // Mod author states
  const [sourceSwfPath, setSourceSwfPath] = useState("")
  const [exportJsonPath, setExportJsonPath] = useState("")

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
          const stats = await stat(selected)
          if (stats.size > MAX_FILE_SIZE) {
            toast.warning(`Large file detected (${Math.round(stats.size / 1024 / 1024)}MB). Processing may take longer.`, {
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
      await invoke("convert_swf_to_json", { swfPath: originalSwfPath, jsonPath: tempJsonPath })

      // Apply the mod's modifications
      await invoke("apply_json_modifications", {
        jsonPath: tempJsonPath,
        modPath: modJsonPath,
        outputJsonPath: tempJsonPath,
      })

      // Convert back to SWF
      await invoke("convert_json_to_swf", {
        jsonPath: tempJsonPath,
        outputPath,
      })

      toast.success("Mod applied successfully!", { id: "apply-mod" })
    } catch (err) {
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

  return (
    <div className="container max-w-3xl">
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

      <div className="mb-8">
        <Logo />
        <p className="text-lg text-center text-muted-foreground mt-2">
          SWF Patcher
        </p>
      </div>

      <Tabs defaultValue="user" className="w-full">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="user">Install Mod</TabsTrigger>
          <TabsTrigger value="author">Create Mod</TabsTrigger>
        </TabsList>

        <TabsContent value="user">
          <Card>
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

        <TabsContent value="author">
          <Card>
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
  )
}
