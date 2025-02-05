import { Routes, Route } from "react-router"
import { ThemeProvider } from "@/components/theme-provider"
import XDeltaPatcher from "@/components/xdelta-patcher"
import SwfPatcher from "@/components/swf-patcher"
import Home from "@/components/home"
import Header from "@/components/header"
import "@/App.css"

export default function App() {
  return (
    <ThemeProvider defaultTheme="dark">
      <div className="absolute inset-0">
        <div className="absolute top-0 z-0 h-screen w-screen bg-neutral-950 bg-custom-gradient"></div>
      </div>
      <div className="relative z-10 min-h-screen">
        <main className="flex min-h-screen flex-col items-center justify-center">
          <Header />
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/binary" element={<XDeltaPatcher />} />
            <Route path="/swf" element={<SwfPatcher />} />
          </Routes>
        </main>
      </div>


    </ThemeProvider>
  )
}
