import { InstallTabs, type ReleaseAssetUrls } from "./InstallTabs"

export const dynamic = "force-dynamic"

const REPO = "Piyush5784/Queryon"
const RELEASES_URL = `https://github.com/${REPO}/releases/latest`

interface GithubReleaseAsset {
  name: string
  browser_download_url: string
}

async function getLatestReleaseAssets(): Promise<ReleaseAssetUrls> {
  const fallback: ReleaseAssetUrls = {
    deb: null,
    rpm: null,
    msi: null,
    exe: null,
    dmgArm64: null,
    dmgX86_64: null,
    releasesUrl: RELEASES_URL,
  }

  try {
    const headers: Record<string, string> = { Accept: "application/vnd.github+json" }
    const token = process.env.GITHUB_RELEASES_TOKEN
    console.error(
      `getLatestReleaseAssets: token present=${!!token} length=${token?.length ?? 0} prefix=${token?.slice(0, 4) ?? "n/a"}`
    )
    if (token) {
      headers.Authorization = `Bearer ${token}`
    }

    const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
      headers,
      next: { revalidate: 3600 },
    })
    if (!res.ok) {
      console.error(
        `getLatestReleaseAssets: GitHub API returned ${res.status} ${res.statusText}`,
        res.headers.get("x-ratelimit-remaining") &&
          `(rate limit remaining: ${res.headers.get("x-ratelimit-remaining")})`
      )
      return fallback
    }

    const release = (await res.json()) as { assets: GithubReleaseAsset[] }
    const find = (predicate: (name: string) => boolean) =>
      release.assets.find((asset) => predicate(asset.name.toLowerCase()))?.browser_download_url ?? null

    return {
      deb: find((name) => name.endsWith(".deb")),
      rpm: find((name) => name.endsWith(".rpm")),
      msi: find((name) => name.endsWith(".msi")),
      exe: find((name) => name.endsWith(".exe")),
      dmgArm64: find((name) => name.endsWith(".dmg") && (name.includes("aarch64") || name.includes("arm64"))),
      dmgX86_64: find((name) => name.endsWith(".dmg") && (name.includes("x86_64") || name.includes("x64"))),
      releasesUrl: RELEASES_URL,
    }
  } catch (err) {
    console.error("getLatestReleaseAssets: fetch failed", err)
    return fallback
  }
}

export default async function InstallPage() {
  const assets = await getLatestReleaseAssets()

  return (
    <div className="flex flex-col gap-8">
      <div>
        <h1 className="text-3xl font-medium tracking-tight">Install Queryon</h1>
        <p className="mt-2 text-muted-foreground">
          Queryon is free and runs on Windows, macOS, and Linux. Pick your platform below.
        </p>
      </div>

      <InstallTabs assets={assets} />
    </div>
  )
}
