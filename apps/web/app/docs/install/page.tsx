"use client"

import { useEffect, useState } from "react"
import { Clock, Download, Terminal } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@queryon/ui/components/alert"
import { Badge } from "@queryon/ui/components/badge"
import { buttonVariants } from "@queryon/ui/components/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@queryon/ui/components/card"
import { Kbd } from "@queryon/ui/components/kbd"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@queryon/ui/components/tabs"
import { cn } from "@queryon/ui/lib/utils"

const RELEASES_URL = "https://github.com/Piyush5784/Queryon/releases/latest"

type Os = "linux" | "windows" | "macos"

function detectOs(): Os {
  if (typeof navigator === "undefined") return "linux"
  const platform = navigator.userAgent.toLowerCase()
  if (platform.includes("win")) return "windows"
  if (platform.includes("mac")) return "macos"
  return "linux"
}

export default function InstallPage() {
  const [os, setOs] = useState<Os>("linux")

  useEffect(() => {
    setOs(detectOs())
  }, [])

  return (
    <div className="flex flex-col gap-8">
      <div>
        <h1 className="text-3xl font-medium tracking-tight">Install Queryon</h1>
        <p className="mt-2 text-muted-foreground">
          Queryon is free and runs on Windows, macOS, and Linux. Pick your platform below.
        </p>
      </div>

      <Tabs value={os} onValueChange={(value) => setOs(value as Os)}>
        <TabsList>
          <TabsTrigger value="linux">Linux</TabsTrigger>
          <TabsTrigger value="windows">Windows</TabsTrigger>
          <TabsTrigger value="macos">macOS</TabsTrigger>
        </TabsList>

        <TabsContent value="linux" className="mt-6">
          <LinuxInstall />
        </TabsContent>
        <TabsContent value="windows" className="mt-6">
          <ComingSoon platform="Windows" />
        </TabsContent>
        <TabsContent value="macos" className="mt-6">
          <ComingSoon platform="macOS" />
        </TabsContent>
      </Tabs>
    </div>
  )
}

function LinuxInstall() {
  return (
    <div className="flex flex-col gap-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Debian / Ubuntu</CardTitle>
            <CardDescription>.deb package</CardDescription>
          </CardHeader>
          <CardContent>
            <a
              href={RELEASES_URL}
              className={cn(buttonVariants({ size: "sm" }), "gap-1.5")}
            >
              <Download className="size-3.5" />
              Download .deb
            </a>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Fedora / RHEL</CardTitle>
            <CardDescription>.rpm package</CardDescription>
          </CardHeader>
          <CardContent>
            <a
              href={RELEASES_URL}
              className={cn(buttonVariants({ size: "sm" }), "gap-1.5")}
            >
              <Download className="size-3.5" />
              Download .rpm
            </a>
          </CardContent>
        </Card>
      </div>

      <div>
        <h2 className="mb-3 flex items-center gap-2 text-lg font-medium">
          <Terminal className="size-4" />
          Install from the terminal
        </h2>
        <div className="flex flex-col gap-3">
          <div>
            <p className="mb-1.5 text-sm text-muted-foreground">Debian / Ubuntu</p>
            <pre className="overflow-x-auto rounded-lg bg-muted px-4 py-3 text-sm">
              <code>sudo dpkg -i Queryon_*.deb</code>
            </pre>
          </div>
          <div>
            <p className="mb-1.5 text-sm text-muted-foreground">Fedora / RHEL</p>
            <pre className="overflow-x-auto rounded-lg bg-muted px-4 py-3 text-sm">
              <code>sudo rpm -i Queryon-*.rpm</code>
            </pre>
          </div>
        </div>
      </div>

      <Alert>
        <Terminal className="size-4" />
        <AlertTitle>First launch</AlertTitle>
        <AlertDescription>
          Find Queryon in your application menu, or run <Kbd>queryon</Kbd> from a terminal.
        </AlertDescription>
      </Alert>
    </div>
  )
}

function ComingSoon({ platform }: { platform: string }) {
  return (
    <Card>
      <CardContent className="flex flex-col items-center gap-3 py-10 text-center">
        <Badge variant="outline" className="gap-1.5">
          <Clock className="size-3" />
          In progress
        </Badge>
        <p className="max-w-sm text-sm text-muted-foreground">
          {platform} builds aren&apos;t published yet. Star the{" "}
          <a
            href="https://github.com/Piyush5784/Queryon"
            className="font-medium text-foreground underline underline-offset-4"
          >
            GitHub repo
          </a>{" "}
          to get notified when they land.
        </p>
      </CardContent>
    </Card>
  )
}
