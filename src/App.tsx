import { BrowserRouter, Routes, Route } from "react-router"
import { Home } from "@/components/home"
import { SwfPatcher } from "@/components/swf-patcher"
import { XdeltaPatcher } from "@/components/xdelta-patcher"
import { ThemeProvider } from "@/components/theme-provider"
import { Toaster } from "@/components/ui/sonner"
import Starfield from "react-starfield"
import "@/App.css"

export default function App() {
  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <BrowserRouter>
        <Starfield
          starCount={3000}
          starColor={[255, 255, 255]}
          speedFactor={0.025}
          backgroundColor="black"
        />
        <div className="absolute inset-0">
          <div className="absolute top-0 z-0 h-screen w-screen bg-neutral-950"></div>
        </div>
        <div className="relative z-10 min-h-screen">
          <main className="flex min-h-screen flex-col items-center justify-center">
            <Routes>
              <Route path="/" element={<Home />} />
              <Route path="/swf-patcher" element={<SwfPatcher />} />
              <Route path="/xdelta-patcher" element={<XdeltaPatcher />} />
            </Routes>
          </main>
        </div>
      </BrowserRouter>
      <Toaster />
    </ThemeProvider>
  )
}
