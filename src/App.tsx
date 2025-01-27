import XDeltaPatcher from "@/components/xdelta-patcher"
import { ThemeProvider } from "@/components/theme-provider"
import "@/App.css"

export const App = () => {
  return (
    <ThemeProvider defaultTheme="dark">
      <div className="absolute inset-0">
        <div className="absolute top-0 z-0 h-screen w-screen bg-neutral-950 bg-custom-gradient"></div>
      </div>
      <main className="relative z-10 flex min-h-screen flex-col items-center justify-center gap-6 p-6 md:p-10">
        <h1 className="text-4xl font-bold font-sg flex items-center">
          <span className="i-arcticons-starfield mr-2"></span>
          StarDelta Patcher
        </h1>
        <XDeltaPatcher />
      </main>
    </ThemeProvider>
  );
};
