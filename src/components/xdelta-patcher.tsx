"use client"

import { useState } from "react"
import { Card, CardContent, CardFooter } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Label } from "@/components/ui/label"
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from "@tauri-apps/api/core"
import { trace, info, error, attachConsole } from '@tauri-apps/plugin-log';

const detach = await attachConsole();

export default function XDeltaPatcher() {
  const [createPatchInputs, setCreatePatchInputs] = useState({
    originalFilePath: "",
    editedFilePath: "",
    outputDir: "",
  })

  const [applyPatchInputs, setApplyPatchInputs] = useState({
    fileToPatchPath: "",
    patchFilePath: "",
    outputDir: "",
  })

  const [result, setResult] = useState<{ success: boolean; message: string } | null>(null)

  const handleCreatePatch = async () => {
    try {
      const { originalFilePath, editedFilePath, outputDir } = createPatchInputs
      trace(`Creating patch with originalFilePath: ${originalFilePath}, editedFilePath: ${editedFilePath}, outputDir: ${outputDir}`)

      if (!originalFilePath || !editedFilePath) {
        setResult({ success: false, message: "Please select both original and edited files." })
        return
      }

      const patch = await invoke('create_patch', {
        args: {
          original_file_path: originalFilePath,
          edited_file_path: editedFilePath,
          output_dir: outputDir,
          original_file_name: originalFilePath.split('\\').pop()
        }
      })
      if (patch) {
        setResult({ success: true, message: "Patch created and saved successfully!" })
        info("Patch created successfully")
      }
    } catch (err: any) {
      setResult({ success: false, message: "Failed to create patch." + err })
      error("Failed to create patch: " + err)
    }
  }

  const handleApplyPatch = async () => {
    try {
      const { fileToPatchPath, patchFilePath, outputDir } = applyPatchInputs
      trace(`Applying patch with fileToPatchPath: ${fileToPatchPath}, patchFilePath: ${patchFilePath}, outputDir: ${outputDir}`)

      if (!fileToPatchPath || !patchFilePath) {
        setResult({ success: false, message: "Please select both the file to patch and the patch file." })
        return
      }

      const decoded = await invoke('apply_patch', {
        args: {
          file_to_patch_path: fileToPatchPath,
          patch_file_path: patchFilePath,
          output_dir: outputDir,
          file_to_patch_name: fileToPatchPath.split('\\').pop()
        }
      })
      if (decoded) {
        setResult({ success: true, message: "Patch applied successfully." })
        info("Patch applied successfully")
      }
    } catch (err: any) {
      setResult({ success: false, message: "Failed to apply patch." + err })
      error("Failed to apply patch: " + err)
    }
  }

  const handleFileSelect = async (operation: "original" | "edited" | "fileToPatch" | "patchFile") => {
    const selectedFile = await open({
      multiple: false,
    });

    if (typeof selectedFile === 'string') {
      if (operation === "original") {
        setCreatePatchInputs((prev) => ({ ...prev, originalFilePath: selectedFile }));
      } else if (operation === "edited") {
        setCreatePatchInputs((prev) => ({ ...prev, editedFilePath: selectedFile }));
      } else if (operation === "fileToPatch") {
        setApplyPatchInputs((prev) => ({ ...prev, fileToPatchPath: selectedFile }));
      } else if (operation === "patchFile") {
        setApplyPatchInputs((prev) => ({ ...prev, patchFilePath: selectedFile }));
      }
    }
  };

  const handleFolderSelect = async (operation: "create" | "apply") => {
    const selectedFolder = await open({
      directory: true,
      multiple: false,
    });

    if (typeof selectedFolder === 'string') {
      if (operation === "create") {
        setCreatePatchInputs((prev) => ({ ...prev, outputDir: selectedFolder }));
      } else {
        setApplyPatchInputs((prev) => ({ ...prev, outputDir: selectedFolder }));
      }
    }
  };

  return (
    <Card className="flex w-full max-w-sm flex-col gap-6 p-6">
      <CardContent>
        <Tabs defaultValue="apply">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="apply">Apply Patch</TabsTrigger>
            <TabsTrigger value="create">Create Patch</TabsTrigger>
          </TabsList>
          <TabsContent value="apply">
            <div className="space-y-4 mt-4">
              <div className="space-y-2">
                <Label htmlFor="fileToPatch">File to Patch</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="fileToPatch"
                    type="text"
                    readOnly
                    value={applyPatchInputs.fileToPatchPath}
                    placeholder="Select file to patch"
                  />
                  <Button type="button" variant="secondary" onClick={() => handleFileSelect("fileToPatch")}>
                    Browse
                  </Button>
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="patchFile">Patch File</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="patchFile"
                    type="text"
                    readOnly
                    value={applyPatchInputs.patchFilePath}
                    placeholder="Select patch file"
                  />
                  <Button type="button" variant="secondary" onClick={() => handleFileSelect("patchFile")}>
                    Browse
                  </Button>
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="applyOutputDir">Output Directory</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="applyOutputDir"
                    type="text"
                    readOnly
                    value={applyPatchInputs.outputDir}
                    placeholder="Select output folder"
                  />
                  <Button type="button" variant="secondary" onClick={() => handleFolderSelect("apply")}>
                    Browse
                  </Button>
                </div>
              </div>
              <Button
                onClick={handleApplyPatch}
                disabled={!applyPatchInputs.fileToPatchPath || !applyPatchInputs.patchFilePath || !applyPatchInputs.outputDir}
              >
                Apply Patch
              </Button>
            </div>
          </TabsContent>
          <TabsContent value="create">
            <div className="space-y-4 mt-4">
              <div className="space-y-2">
                <Label htmlFor="originalFile">Original File</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="originalFile"
                    type="text"
                    readOnly
                    value={createPatchInputs.originalFilePath}
                    placeholder="Select original file"
                  />
                  <Button type="button" variant="secondary" onClick={() => handleFileSelect("original")}>
                    Browse
                  </Button>
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="editedFile">Edited File</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="editedFile"
                    type="text"
                    readOnly
                    value={createPatchInputs.editedFilePath}
                    placeholder="Select edited file"
                  />
                  <Button type="button" variant="secondary" onClick={() => handleFileSelect("edited")}>
                    Browse
                  </Button>
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="createOutputDir">Output Directory</Label>
                <div className="flex items-center space-x-2">
                  <Input
                    id="createOutputDir"
                    type="text"
                    readOnly
                    value={createPatchInputs.outputDir}
                    placeholder="Select output folder"
                  />
                  <Button type="button" variant="secondary" onClick={() => handleFolderSelect("create")}>
                    Browse
                  </Button>
                </div>
              </div>
              <Button
                onClick={handleCreatePatch}
                disabled={!createPatchInputs.originalFilePath || !createPatchInputs.editedFilePath || !createPatchInputs.outputDir}
              >
                Create Patch
              </Button>
            </div>
          </TabsContent>
        </Tabs>
      </CardContent>
      <CardFooter>
        {result && (
          <Alert variant={result.success ? "default" : "destructive"}>
            <AlertTitle>{result.success ? "Success" : "Error"}</AlertTitle>
            <AlertDescription>{result.message}</AlertDescription>
          </Alert>
        )}
      </CardFooter>
    </Card>
  )
}

detach();
