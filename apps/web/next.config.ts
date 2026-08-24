import type { NextConfig } from "next"
import createMDX from "@next/mdx"

const nextConfig: NextConfig = {
  transpilePackages: ["@queryon/ui"],
  pageExtensions: ["ts", "tsx", "mdx"],
}

const withMDX = createMDX({})

export default withMDX(nextConfig)
