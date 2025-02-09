"use client"

import { useState, useEffect, useRef } from "react"
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
import { Window, PhysicalSize } from "@tauri-apps/api/window"

const MAX_FILE_SIZE = 100 * 1024 * 1024 // 100MB
const PATCH_EXTENSIONS = ["vcdiff", "xdelta", "xdelta3"]

export function XdeltaPatcher() {
  // End-user states
  const [originalPath, setOriginalPath] = useState("")
  const [patchPath, setPatchPath] = useState("")
  const [outputPath, setOutputPath] = useState("")
  const [isApplyingPatch, setIsApplyingPatch] = useState(false)
  const [isCreatingPatch, setIsCreatingPatch] = useState(false)

  // Mod author states
  const [sourcePath, setSourcePath] = useState("")
  const [modifiedPath, setModifiedPath] = useState("")
  const [exportPatchPath, setExportPatchPath] = useState("")

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
    filters?: { name: string; extensions: string[] }[]
  ) => {
    try {
      const selected = await open({
        multiple: false,
        filters,
      })
      if (selected && typeof selected === "string") {
        // Check file extension if filters are provided
        if (filters?.length) {
          const ext = selected.split('.').pop()?.toLowerCase()
          if (!filters[0].extensions.includes(ext || '')) {
            toast.error(`Invalid file type. Expected ${filters[0].extensions.map(e => `.${e}`).join(" or ")} file`)
            return
          }
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

  const handleApplyPatch = async () => {
    try {
      if (!originalPath) throw new Error("Please select the original file")
      if (!patchPath) throw new Error("Please select the patch file")
      if (!outputPath) throw new Error("Please specify where to save the patched file")

      setIsApplyingPatch(true)
      toast.loading("Applying patch...", { id: "apply-patch" })

      await invoke("apply_patch", {
        originalPath,
        patchPath,
        outputPath,
      })

      toast.success("Patch applied successfully!", { id: "apply-patch" })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to apply patch", { id: "apply-patch" })
    } finally {
      setIsApplyingPatch(false)
    }
  }

  const handleCreatePatch = async () => {
    try {
      if (!sourcePath) throw new Error("Please select the source file")
      if (!modifiedPath) throw new Error("Please select the modified file")
      if (!exportPatchPath) throw new Error("Please specify where to save the patch")

      setIsCreatingPatch(true)
      toast.loading("Creating patch...", { id: "create-patch" })

      await invoke("create_patch", {
        sourcePath,
        targetPath: modifiedPath,
        patchPath: exportPatchPath,
      })

      toast.success("Patch created successfully!", { id: "create-patch" })
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to create patch", { id: "create-patch" })
    } finally {
      setIsCreatingPatch(false)
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
          <Tabs defaultValue="user">
            <TabsList className="grid grid-cols-2 mb-4">
              <TabsTrigger value="user">Apply Patch</TabsTrigger>
              <TabsTrigger value="author">Create Patch</TabsTrigger>
            </TabsList>

            <TabsContent value="user">
              <Card className="min-h-[400px]">
                <CardHeader>
                  <CardTitle>Apply Binary Patch</CardTitle>
                  <CardDescription>
                    Apply an xdelta3 patch to your original file
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-2">
                    <Label htmlFor="original-file">Original File</Label>
                    <div className="flex gap-2">
                      <Input
                        id="original-file"
                        value={originalPath}
                        readOnly
                        placeholder="Select your original file..."
                      />
                      <Button
                        onClick={() => handleSelectFile(setOriginalPath)}
                      >
                        Browse
                      </Button>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="patch-file">Patch File</Label>
                    <div className="flex gap-2">
                      <Input
                        id="patch-file"
                        value={patchPath}
                        readOnly
                        placeholder="Select the patch file..."
                      />
                      <Button
                        onClick={() =>
                          handleSelectFile(setPatchPath, [
                            { name: "xDelta3 Patch", extensions: PATCH_EXTENSIONS },
                          ])
                        }
                      >
                        Browse
                      </Button>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="output-file">Save Patched File As</Label>
                    <div className="flex gap-2">
                      <Input
                        id="output-file"
                        value={outputPath}
                        readOnly
                        placeholder="Choose where to save the patched file..."
                      />
                      <Button
                        onClick={() => handleSaveFile(setOutputPath, [])}
                      >
                        Browse
                      </Button>
                    </div>
                  </div>

                  <Button
                    className="w-full"
                    onClick={handleApplyPatch}
                    disabled={!originalPath || !patchPath || !outputPath || isApplyingPatch}
                  >
                    {isApplyingPatch ? "Applying Patch..." : "Apply Patch"}
                  </Button>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="author">
              <Card className="min-h-[400px]">
                <CardHeader>
                  <CardTitle>Create Binary Patch</CardTitle>
                  <CardDescription>
                    Create an xdelta3 patch by comparing original and modified files
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-6">
                  <div className="rounded-lg border p-4">
                    <h3 className="font-medium mb-2">Create Patch</h3>
                    <p className="text-sm text-muted-foreground mb-4">
                      Select the original and modified files to generate a patch that can transform one into the other.
                    </p>
                    <div className="space-y-4">
                      <div className="flex gap-2">
                        <Input
                          value={sourcePath}
                          readOnly
                          placeholder="Select original file..."
                        />
                        <Button
                          onClick={() => handleSelectFile(setSourcePath)}
                        >
                          Browse
                        </Button>
                      </div>
                      <div className="flex gap-2">
                        <Input
                          value={modifiedPath}
                          readOnly
                          placeholder="Select modified file..."
                        />
                        <Button
                          onClick={() => handleSelectFile(setModifiedPath)}
                        >
                          Browse
                        </Button>
                      </div>
                      <div className="flex gap-2">
                        <Input
                          value={exportPatchPath}
                          readOnly
                          placeholder="Save patch file as..."
                        />
                        <Button
                          onClick={() =>
                            handleSaveFile(setExportPatchPath, [
                              { name: "xDelta3 Patch", extensions: [PATCH_EXTENSIONS[0]] },
                            ])
                          }
                        >
                          Browse
                        </Button>
                      </div>
                      <div className="flex flex-col gap-2">
                        <Button
                          onClick={handleCreatePatch}
                          disabled={!sourcePath || !modifiedPath || !exportPatchPath || isCreatingPatch}
                        >
                          {isCreatingPatch ? "Creating Patch..." : "Create Patch"}
                        </Button>
                        <p className="text-xs text-muted-foreground text-center">
                          The patch will contain only the differences between files. Users will need their own copy of the original file to apply the patch.
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
