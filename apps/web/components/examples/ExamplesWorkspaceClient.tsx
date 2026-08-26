"use client"

import dynamic from "next/dynamic"

import { WindowChrome } from "../landing/mocks/WindowChrome"

const ExamplesWorkspace = dynamic(
  () => import("./ExamplesWorkspace").then((mod) => mod.ExamplesWorkspace),
  {
    ssr: false,
    loading: () => (
      <WindowChrome title="devdb — Queryon">
        <div className="flex h-[640px] items-center justify-center text-sm text-muted-foreground">
          Loading grid…
        </div>
      </WindowChrome>
    ),
  }
)

export function ExamplesWorkspaceClient() {
  return <ExamplesWorkspace />
}
