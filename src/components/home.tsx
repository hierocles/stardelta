"use client"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { NavLink } from "react-router"
import { Logo } from "@/components/ui/logo"

export function Home() {
  return (
    <div className="container flex flex-col items-center gap-8 py-8">
      <div className="text-center">
        <Logo />
        <p className="text-lg text-muted-foreground mt-2">
          A tool for modifying Starfield UI files
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
      <Card className="flex flex-col">
          <CardHeader>
            <CardTitle>SWF Patcher</CardTitle>
            <CardDescription>
              Modify SWF files with SVG shape replacements and other customizations
            </CardDescription>
          </CardHeader>
          <CardContent className="flex-1">
            <NavLink
              to="/swf-patcher"
              className="inline-flex h-10 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground ring-offset-background transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
            >
              Open SWF Patcher
            </NavLink>
          </CardContent>
        </Card>
        <Card className="flex flex-col">
          <CardHeader>
            <CardTitle>Binary Patcher</CardTitle>
            <CardDescription>
              Create and apply xdelta3 patches for any file
            </CardDescription>
          </CardHeader>
          <CardContent className="flex-1">
            <NavLink
              to="/xdelta-patcher"
              className="inline-flex h-10 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground ring-offset-background transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
            >
              Open Binary Patcher
            </NavLink>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
