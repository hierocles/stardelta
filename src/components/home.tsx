import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Link } from "react-router"

export default function Home() {
  return (
    <Card className="w-full max-w-sm">
        <CardContent className="flex flex-col gap-4 p-6">
            <Link to="/binary" className="w-full">
            <Button className="w-full h-20 text-lg" variant="outline">
                Binary Patch
                <span className="block text-sm text-muted-foreground mt-1">
                Create or apply xdelta patches
                </span>
            </Button>
            </Link>
            <Link to="/yaml" className="w-full">
            <Button className="w-full h-20 text-lg" variant="outline">
                YAML Patch
                <span className="block text-sm text-muted-foreground mt-1">
                Patch SWF files using YAML config
                </span>
            </Button>
            </Link>
        </CardContent>
    </Card>
  )
}
