"use client"

import { useEffect, useState } from "react"
import { Download, ExternalLink, Terminal } from "lucide-react"

import { Alert, AlertDescription, AlertTitle } from "@queryon/ui/components/alert"
import { buttonVariants } from "@queryon/ui/components/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@queryon/ui/components/card"
import { Kbd } from "@queryon/ui/components/kbd"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@queryon/ui/components/tabs"
import { cn } from "@queryon/ui/lib/utils"

export interface ReleaseAssetUrls {
  deb: string | null
  rpm: string | null
  msi: string | null
  exe: string | null
  dmgArm64: string | null
  dmgX86_64: string | null
  releasesUrl: string
}

type Os = "linux" | "windows" | "macos"

function detectOs(): Os {
  if (typeof navigator === "undefined") return "linux"
  const platform = navigator.userAgent.toLowerCase()
  if (platform.includes("win")) return "windows"
  if (platform.includes("mac")) return "macos"
  return "linux"
}

export function InstallTabs({ assets }: { assets: ReleaseAssetUrls }) {
  const [os, setOs] = useState<Os>("linux")

  useEffect(() => {
    setOs(detectOs())
  }, [])

  return (
    <Tabs value={os} onValueChange={(value) => setOs(value as Os)}>
      <TabsList>
        <TabsTrigger value="linux">Linux</TabsTrigger>
        <TabsTrigger value="windows">Windows</TabsTrigger>
        <TabsTrigger value="macos">macOS</TabsTrigger>
      </TabsList>

      <TabsContent value="linux" className="mt-6">
        <LinuxInstall assets={assets} />
      </TabsContent>
      <TabsContent value="windows" className="mt-6">
        <WindowsInstall assets={assets} />
      </TabsContent>
      <TabsContent value="macos" className="mt-6">
        <MacInstall assets={assets} />
      </TabsContent>
    </Tabs>
  )
}

function DownloadButton({ href, label }: { href: string | null; label: string }) {
  if (!href) {
    return (
      <span className={cn(buttonVariants({ size: "sm", variant: "outline" }), "gap-1.5 opacity-60")}>
        {label} unavailable
      </span>
    )
  }
  return (
    <a href={href} className={cn(buttonVariants({ size: "sm" }), "gap-1.5")}>
      <Download className="size-3.5" />
      {label}
    </a>
  )
}

function AllReleasesLink({ releasesUrl }: { releasesUrl: string }) {
  return (
    <a
      href={releasesUrl}
      className="inline-flex items-center gap-1 text-sm text-muted-foreground underline underline-offset-4 hover:text-foreground"
    >
      View all releases on GitHub
      <ExternalLink className="size-3.5" />
    </a>
  )
}

function LinuxInstall({ assets }: { assets: ReleaseAssetUrls }) {
  return (
    <div className="flex flex-col gap-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Debian / Ubuntu</CardTitle>
            <CardDescription>.deb package</CardDescription>
          </CardHeader>
          <CardContent>
            <DownloadButton href={assets.deb} label="Download .deb" />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Fedora / RHEL</CardTitle>
            <CardDescription>.rpm package</CardDescription>
          </CardHeader>
          <CardContent>
            <DownloadButton href={assets.rpm} label="Download .rpm" />
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

      <AllReleasesLink releasesUrl={assets.releasesUrl} />
    </div>
  )
}

function WindowsInstall({ assets }: { assets: ReleaseAssetUrls }) {
  return (
    <div className="flex flex-col gap-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Installer (recommended)</CardTitle>
            <CardDescription>.msi package</CardDescription>
          </CardHeader>
          <CardContent>
            <DownloadButton href={assets.msi} label="Download .msi" />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Setup executable</CardTitle>
            <CardDescription>.exe installer</CardDescription>
          </CardHeader>
          <CardContent>
            <DownloadButton href={assets.exe} label="Download .exe" />
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

      <AllReleasesLink releasesUrl={assets.releasesUrl} />
    </div>
  )
}

function MacInstall({ assets }: { assets: ReleaseAssetUrls }) {
  return (
    <div className="flex flex-col gap-6">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Apple Silicon</CardTitle>
            <CardDescription>M1/M2/M3/M4 Macs</CardDescription>
          </CardHeader>
          <CardContent>
            <DownloadButton href={assets.dmgArm64} label="Download .dmg" />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Intel</CardTitle>
            <CardDescription>Intel-based Macs</CardDescription>
          </CardHeader>
          <CardContent>
            <DownloadButton href={assets.dmgX86_64} label="Download .dmg" />
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

      <AllReleasesLink releasesUrl={assets.releasesUrl} />
    </div>
  )
}
