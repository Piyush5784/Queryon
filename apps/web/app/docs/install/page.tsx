"use client"

import { useEffect, useState } from "react"
import { Download, Terminal } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@queryon/ui/components/alert"
import { buttonVariants } from "@queryon/ui/components/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@queryon/ui/components/card"
import { Kbd } from "@queryon/ui/components/kbd"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@queryon/ui/components/tabs"
import { cn } from "@queryon/ui/lib/utils"

const R2_BASE_URL = "https://pub-16a98e553e3d4f7db28008ca9262706f.r2.dev"
const DEB_URL = `${R2_BASE_URL}/latest/queryon.deb`
const RPM_URL = `${R2_BASE_URL}/latest/queryon.rpm`
const MSI_URL = `${R2_BASE_URL}/latest/queryon.msi`
const EXE_URL = `${R2_BASE_URL}/latest/queryon.exe`
const DMG_ARM64_URL = `${R2_BASE_URL}/latest/queryon-arm64.dmg`
const DMG_X86_64_URL = `${R2_BASE_URL}/latest/queryon-x86_64.dmg`

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
          <WindowsInstall />
        </TabsContent>
        <TabsContent value="macos" className="mt-6">
          <MacInstall />
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
              href={DEB_URL}
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
              href={RPM_URL}
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

function WindowsInstall() {
  return (
    <div className="flex flex-col gap-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Installer (recommended)</CardTitle>
            <CardDescription>.msi package</CardDescription>
          </CardHeader>
          <CardContent>
            <a href={MSI_URL} className={cn(buttonVariants({ size: "sm" }), "gap-1.5")}>
              <Download className="size-3.5" />
              Download .msi
            </a>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Setup executable</CardTitle>
            <CardDescription>.exe installer</CardDescription>
          </CardHeader>
          <CardContent>
            <a href={EXE_URL} className={cn(buttonVariants({ size: "sm" }), "gap-1.5")}>
              <Download className="size-3.5" />
              Download .exe
            </a>
          </CardContent>
        </Card>
      </div>

      <Alert>
        <Terminal className="size-4" />
        <AlertTitle>First launch</AlertTitle>
        <AlertDescription>
          Windows SmartScreen may warn about an unrecognized app. Click{" "}
          <Kbd>More info</Kbd> then <Kbd>Run anyway</Kbd> to continue — this is expected
          for apps without a paid code-signing certificate.
        </AlertDescription>
      </Alert>
    </div>
  )
}

function MacInstall() {
  return (
    <div className="flex flex-col gap-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Apple Silicon</CardTitle>
            <CardDescription>M1/M2/M3/M4 Macs</CardDescription>
          </CardHeader>
          <CardContent>
            <a href={DMG_ARM64_URL} className={cn(buttonVariants({ size: "sm" }), "gap-1.5")}>
              <Download className="size-3.5" />
              Download .dmg
            </a>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Intel</CardTitle>
            <CardDescription>Intel-based Macs</CardDescription>
          </CardHeader>
          <CardContent>
            <a href={DMG_X86_64_URL} className={cn(buttonVariants({ size: "sm" }), "gap-1.5")}>
              <Download className="size-3.5" />
              Download .dmg
            </a>
          </CardContent>
        </Card>
      </div>

      <Alert>
        <Terminal className="size-4" />
        <AlertTitle>First launch</AlertTitle>
        <AlertDescription>
          macOS Gatekeeper may block the app since it isn&apos;t notarized yet. Right-click
          the app in Finder, choose <Kbd>Open</Kbd>, then confirm <Kbd>Open</Kbd> in the
          dialog that appears.
        </AlertDescription>
      </Alert>
    </div>
  )
}
