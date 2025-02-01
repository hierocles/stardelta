"use client"

import { useState } from "react"
import { Card, CardContent, CardFooter } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Label } from "@/components/ui/label"
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from "@tauri-apps/api/core"
import { NavLink } from "react-router"

export default function YamlPatcher() {
  const [patchInputs, setPatchInputs] = useState({
    swfFilePath: "",
    yamlFilePath: "",
    outputDir: "",
  })

  const [result, setResult] = useState<{ success: boolean; message: string } | null>(null)

  const handlePatch = async () => {
    try {
      const { swfFilePath, yamlFilePath, outputDir } = patchInputs
      console.log(`Patching SWF with config: ${yamlFilePath}, swfPath: ${swfFilePath}, outputDir: ${outputDir}`)

      if (!swfFilePath || !yamlFilePath) {
        setResult({ success: false, message: "Please select both SWF and YAML files." })
        return
      }

      const patched = await invoke('patch_swf', {
        args: {
          swf_file_path: swfFilePath,
          yaml_file_path: yamlFilePath,
          output_dir: outputDir,
          swf_file_name: swfFilePath.split('\\').pop()
        }
      })
      if (patched) {
        setResult({ success: true, message: "SWF patched successfully!" })
        console.info("SWF patched successfully")
      }
    } catch (err: any) {
      setResult({ success: false, message: "Failed to patch SWF: " + err })
      console.error("Failed to patch SWF: " + err)
    }
  }

  const handleFileSelect = async (operation: "swf" | "yaml") => {
    const selectedFile = await open({
      multiple: false,
      filters: [{
        name: operation === "swf" ? "SWF Files" : "YAML Files",
        extensions: [operation === "swf" ? "swf" : "yml", operation === "yaml" ? "yaml" : ""]
      }]
    });

    if (typeof selectedFile === 'string') {
      if (operation === "swf") {
        setPatchInputs((prev) => ({ ...prev, swfFilePath: selectedFile }));
      } else if (operation === "yaml") {
        setPatchInputs((prev) => ({ ...prev, yamlFilePath: selectedFile }));
      }
    }
  };

  const handleFolderSelect = async () => {
    const selectedFolder = await open({
      directory: true,
      multiple: false,
    });

    if (typeof selectedFolder === 'string') {
      setPatchInputs((prev) => ({ ...prev, outputDir: selectedFolder }));
    }
  };

  return (
    <div className="flex flex-col items-center gap-4">
      <NavLink
        to="/"
        className="self-start rounded-lg border px-4 py-2 text-sm hover:bg-accent"
      >
        ← Back
      </NavLink>
      <Card className="flex w-full max-w-sm flex-col gap-6 p-6">
        <CardContent>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="swfFile">SWF File</Label>
              <div className="flex items-center space-x-2">
                <Input
                  id="swfFile"
                  type="text"
                  readOnly
                  value={patchInputs.swfFilePath}
                  placeholder="Select SWF file"
                />
                <Button type="button" variant="secondary" onClick={() => handleFileSelect("swf")}>
                  Browse
                </Button>
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="yamlFile">YAML Configuration</Label>
              <div className="flex items-center space-x-2">
                <Input
                  id="yamlFile"
                  type="text"
                  readOnly
                  value={patchInputs.yamlFilePath}
                  placeholder="Select YAML file"
                />
                <Button type="button" variant="secondary" onClick={() => handleFileSelect("yaml")}>
                  Browse
                </Button>
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="outputDir">Output Directory</Label>
              <div className="flex items-center space-x-2">
                <Input
                  id="outputDir"
                  type="text"
                  readOnly
                  value={patchInputs.outputDir}
                  placeholder="Select output folder"
                />
                <Button type="button" variant="secondary" onClick={handleFolderSelect}>
                  Browse
                </Button>
              </div>
            </div>
            <Button
              onClick={handlePatch}
              disabled={!patchInputs.swfFilePath || !patchInputs.yamlFilePath || !patchInputs.outputDir}
            >
              Patch SWF
            </Button>
          </div>
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
    </div>
  )
}
