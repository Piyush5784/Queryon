"use client"

import { useEffect, useRef } from "react"
import { useTheme } from "next-themes"
import * as THREE from "three"

import { isWebGLAvailable } from "./HeroScene"

const COUNT = 3200
const SHAPES = 4

type Point = { p: [number, number, number]; accent: number; row?: number; speed?: number }

const vertex = `
  attribute vec3 aP0;
  attribute vec3 aP1;
  attribute vec3 aP2;
  attribute vec3 aP3;
  attribute vec4 aAccent;
  attribute float aRow;
  attribute float aSpeed;
  attribute float aSeed;
  uniform vec4 uW;
  uniform float uTime;
  uniform float uScale;
  varying float vGlow;
  varying float vAlpha;

  vec3 spinY(vec3 p, float a) {
    float c = cos(a);
    float s = sin(a);
    return vec3(c * p.x + s * p.z, p.y, -s * p.x + c * p.z);
  }

  void main() {
    vec3 p0 = spinY(aP0, uTime * 0.14);
    vec3 p1 = aP1;
    p1.x = mod(p1.x + uTime * aSpeed + 5.5, 11.0) - 5.5;
    vec3 p2 = aP2;
    p2.z += sin(aP2.x * 0.7 + uTime * 0.9) * 0.12;
    vec3 p3 = aP3;
    p3.y += sin(uTime * 0.8 + aP3.x * 0.6) * 0.06;

    vec3 p = p0 * uW.x + p1 * uW.y + p2 * uW.z + p3 * uW.w;
    float settle = max(max(uW.x, uW.y), max(uW.z, uW.w));
    float burst = (1.0 - settle) * 2.2;
    p += vec3(sin(aSeed * 91.0 + uTime * 1.3), cos(aSeed * 47.0 + uTime), sin(aSeed * 13.0 - uTime * 0.7)) * burst;

    float scan = fract(uTime * 0.12);
    float band = exp(-pow((aRow - scan) * 14.0, 2.0)) * uW.z;
    float pulse = (0.5 + 0.5 * sin(uTime * 2.2 + aSeed * 70.0)) * 0.22;
    float accent = dot(aAccent, uW);
    vGlow = 0.32 + pulse + accent * 0.75 + band * 1.1;
    vAlpha = mix(1.0, smoothstep(5.5, 3.6, abs(p1.x)), uW.y);

    vec4 mv = modelViewMatrix * vec4(p, 1.0);
    gl_PointSize = uScale * (1.25 + accent * 1.5 + band * 1.4) * (8.0 / -mv.z);
    gl_Position = projectionMatrix * mv;
  }
`

const fragment = `
  uniform float uLight;
  varying float vGlow;
  varying float vAlpha;
  void main() {
    float d = length(gl_PointCoord - 0.5);
    if (d > 0.5) discard;
    float alpha = smoothstep(0.5, 0.05, d) * min(vGlow, 1.0) * vAlpha;
    vec3 dark = mix(vec3(0.13, 0.77, 0.37), vec3(0.8, 1.0, 0.88), clamp(vGlow - 0.75, 0.0, 1.0)) * vGlow;
    vec3 light = mix(vec3(0.2, 0.72, 0.4), vec3(0.03, 0.4, 0.17), clamp(vGlow - 0.6, 0.0, 1.0));
    gl_FragColor = vec4(mix(dark, light, uLight), alpha * mix(1.0, 0.8, uLight));
  }
`

function fit(points: Point[]): Point[] {
  if (points.length >= COUNT) {
    const step = points.length / COUNT
    return Array.from({ length: COUNT }, (_, i) => points[Math.floor(i * step)]!)
  }
  const out = [...points]
  while (out.length < COUNT) {
    const src = points[Math.floor(Math.random() * points.length)]!
    out.push({ ...src, p: [src.p[0] + (Math.random() - 0.5) * 0.04, src.p[1] + (Math.random() - 0.5) * 0.04, src.p[2]] })
  }
  return out
}

function line(from: [number, number, number], to: [number, number, number], spacing: number, accent: number): Point[] {
  const length = Math.hypot(to[0] - from[0], to[1] - from[1], to[2] - from[2])
  const steps = Math.max(1, Math.round(length / spacing))
  return Array.from({ length: steps + 1 }, (_, i) => {
    const t = i / steps
    return { p: [from[0] + (to[0] - from[0]) * t, from[1] + (to[1] - from[1]) * t, from[2] + (to[2] - from[2]) * t], accent }
  })
}

function networkShape(): Point[] {
  const points: Point[] = []
  const sphere = 1700
  const golden = Math.PI * (3 - Math.sqrt(5))
  for (let i = 0; i < sphere; i++) {
    const y = 1 - (i / (sphere - 1)) * 2
    const r = Math.sqrt(1 - y * y)
    const theta = golden * i
    points.push({ p: [Math.cos(theta) * r * 2.5, y * 2.5, Math.sin(theta) * r * 2.5], accent: 0 })
  }
  const rings = [
    { radius: 3.4, tilt: 1.2, roll: 0.25 },
    { radius: 3.9, tilt: 1.35, roll: -0.4 },
  ]
  rings.forEach(({ radius, tilt, roll }) => {
    const ring = new THREE.Euler(tilt, 0, roll)
    for (let i = 0; i < 360; i++) {
      const a = (i / 360) * Math.PI * 2
      const v = new THREE.Vector3(Math.cos(a) * radius, Math.sin(a) * radius, 0).applyEuler(ring)
      points.push({ p: [v.x, v.y, v.z], accent: 0.15 })
    }
    for (let n = 0; n < 4; n++) {
      const a = (n / 4) * Math.PI * 2 + tilt
      const center = new THREE.Vector3(Math.cos(a) * radius, Math.sin(a) * radius, 0).applyEuler(ring)
      for (let k = 0; k < 40; k++) {
        const dir = new THREE.Vector3(Math.random() - 0.5, Math.random() - 0.5, Math.random() - 0.5).normalize().multiplyScalar(0.16)
        points.push({ p: [center.x + dir.x, center.y + dir.y, center.z + dir.z], accent: 1 })
      }
    }
  })
  return points
}

function streamShape(): Point[] {
  const points: Point[] = []
  const lines = 30
  for (let l = 0; l < lines; l++) {
    const y = -2.7 + (l / (lines - 1)) * 5.4
    const z = -2.4 + Math.random() * 3.4
    const speed = 0.5 + Math.random() * 1.6
    let x = -5.5 + Math.random() * 0.6
    while (x < 5.5) {
      const width = 0.25 + Math.random() * 1.1
      const keyword = Math.random() < 0.28 ? 1 : 0
      for (let px = x; px < Math.min(x + width, 5.5); px += 0.055) {
        points.push({ p: [px, y, z], accent: keyword, speed })
      }
      x += width + 0.18 + Math.random() * 0.3
    }
  }
  return points
}

function gridShape(): Point[] {
  const points: Point[] = []
  const rows = 15
  const cols = 7
  const tilt = 0.95
  const width = 8.6
  const depth = 7.2
  for (let r = 0; r < rows; r++) {
    const v = -depth / 2 + (r / (rows - 1)) * depth
    const y = v * Math.cos(tilt) - 0.2
    const z = -v * Math.sin(tilt)
    const header = r === rows - 1 ? 1 : 0
    for (let c = 0; c < cols; c++) {
      const cellStart = -width / 2 + (c / cols) * width + 0.12
      const textWidth = (width / cols) * (header ? 0.55 : 0.25 + Math.random() * 0.5)
      for (let px = 0; px < textWidth; px += 0.05) {
        points.push({ p: [cellStart + px, y, z], accent: header, row: r / (rows - 1) })
      }
    }
    for (let px = -width / 2; px <= width / 2; px += 0.11) {
      points.push({ p: [px, y - 0.12 * Math.cos(tilt), z + 0.24 * Math.sin(tilt)], accent: 0, row: r / (rows - 1) })
    }
  }
  return points
}

function schemaShape(): Point[] {
  const points: Point[] = []
  const tables = [
    { x: -2.9, y: 1.1, z: -0.4, w: 2.1, h: 1.8 },
    { x: 1.1, y: 1.6, z: 0.4, w: 2.3, h: 1.4 },
    { x: -1.6, y: -1.6, z: 0.7, w: 2.0, h: 1.5 },
    { x: 2.0, y: -1.2, z: -0.9, w: 2.1, h: 1.7 },
  ]
  tables.forEach(({ x, y, z, w, h }) => {
    const left = x - w / 2
    const right = x + w / 2
    const top = y + h / 2
    const bottom = y - h / 2
    points.push(...line([left, top, z], [right, top, z], 0.045, 0.6))
    points.push(...line([right, top, z], [right, bottom, z], 0.045, 0.6))
    points.push(...line([right, bottom, z], [left, bottom, z], 0.045, 0.6))
    points.push(...line([left, bottom, z], [left, top, z], 0.045, 0.6))
    points.push(...line([left, top - 0.38, z], [right, top - 0.38, z], 0.045, 0.6))
    for (let px = left + 0.18; px < left + w * 0.55; px += 0.045) points.push({ p: [px, top - 0.19, z], accent: 1 })
    for (let r = top - 0.62; r > bottom + 0.12; r -= 0.28) {
      const len = w * (0.3 + Math.random() * 0.4)
      for (let px = left + 0.18; px < left + 0.18 + len; px += 0.05) points.push({ p: [px, r, z], accent: 0 })
    }
  })
  const links: [number, number][] = [
    [0, 1],
    [0, 2],
    [1, 3],
    [2, 3],
  ]
  links.forEach(([a, b]) => {
    const from = tables[a]!
    const to = tables[b]!
    points.push(...line([from.x, from.y, from.z], [to.x, to.y, to.z], 0.12, 0.35))
  })
  return points
}

interface FeatureSceneProps {
  active: number
}

export function FeatureScene({ active }: FeatureSceneProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const activeRef = useRef(active)
  const redrawRef = useRef<(() => void) | null>(null)
  const themeRef = useRef<((light: boolean) => void) | null>(null)
  const { resolvedTheme } = useTheme()
  const light = resolvedTheme !== "dark"
  const lightRef = useRef(light)

  useEffect(() => {
    activeRef.current = active
    redrawRef.current?.()
  }, [active])

  useEffect(() => {
    lightRef.current = light
    themeRef.current?.(light)
  }, [light])

  useEffect(() => {
    const container = containerRef.current
    if (!container || !isWebGLAvailable()) return

    let renderer: THREE.WebGLRenderer
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: "high-performance" })
    } catch {
      return
    }
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches
    const dpr = Math.min(window.devicePixelRatio, 2)
    renderer.setPixelRatio(dpr)
    renderer.setClearColor(0x000000, 0)
    container.appendChild(renderer.domElement)

    const scene = new THREE.Scene()
    const camera = new THREE.PerspectiveCamera(35, 1, 0.1, 100)
    camera.position.set(0, 0, 10)
    const rig = new THREE.Group()
    scene.add(rig)

    const shapes = [networkShape(), streamShape(), gridShape(), schemaShape()].map(fit)
    const geometry = new THREE.BufferGeometry()
    shapes.forEach((shape, s) => {
      geometry.setAttribute(`aP${s}`, new THREE.Float32BufferAttribute(shape.flatMap((pt) => pt.p), 3))
    })
    geometry.setAttribute("position", geometry.getAttribute("aP0"))
    geometry.setAttribute(
      "aAccent",
      new THREE.Float32BufferAttribute(Array.from({ length: COUNT }, (_, i) => shapes.map((shape) => shape[i]!.accent)).flat(), 4)
    )
    geometry.setAttribute("aRow", new THREE.Float32BufferAttribute(shapes[2]!.map((pt) => pt.row ?? -1), 1))
    geometry.setAttribute("aSpeed", new THREE.Float32BufferAttribute(shapes[1]!.map((pt) => pt.speed ?? 1), 1))
    geometry.setAttribute("aSeed", new THREE.Float32BufferAttribute(Array.from({ length: COUNT }, () => Math.random()), 1))
    geometry.boundingSphere = new THREE.Sphere(new THREE.Vector3(), 12)

    const weights = new THREE.Vector4(1, 0, 0, 0)
    const uniforms = { uW: { value: weights }, uTime: { value: 0 }, uScale: { value: 1 }, uLight: { value: 0 } }
    const material = new THREE.ShaderMaterial({
      vertexShader: vertex,
      fragmentShader: fragment,
      uniforms,
      transparent: true,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    })
    rig.add(new THREE.Points(geometry, material))

    const applyTheme = (isLight: boolean) => {
      uniforms.uLight.value = isLight ? 1 : 0
      material.blending = isLight ? THREE.NormalBlending : THREE.AdditiveBlending
      material.needsUpdate = true
    }
    applyTheme(lightRef.current)

    const current = [1, 0, 0, 0]
    const setTarget = (ease: number) => {
      for (let s = 0; s < SHAPES; s++) {
        const target = s === activeRef.current ? 1 : 0
        current[s]! += (target - current[s]!) * ease
      }
      const total = current.reduce((sum, w) => sum + w, 0) || 1
      weights.set(current[0]! / total, current[1]! / total, current[2]! / total, current[3]! / total)
    }

    const resize = () => {
      const { clientWidth: w, clientHeight: h } = container
      if (!w || !h) return
      renderer.setSize(w, h, false)
      camera.aspect = w / h
      camera.position.z = camera.aspect < 1 ? 12.5 : 10
      camera.updateProjectionMatrix()
      uniforms.uScale.value = (h * dpr) / 560
    }

    const pointer = { x: 0, y: 0, tx: 0, ty: 0 }
    const onPointer = (event: PointerEvent) => {
      const rect = container.getBoundingClientRect()
      pointer.tx = (event.clientX - rect.left) / rect.width - 0.5
      pointer.ty = (event.clientY - rect.top) / rect.height - 0.5
    }

    const frame = (t: number, dt: number) => {
      uniforms.uTime.value = t
      setTarget(dt === 0 ? 1 : 1 - Math.pow(0.02, dt))
      const ease = dt === 0 ? 1 : 1 - Math.pow(0.05, dt)
      pointer.x += (pointer.tx - pointer.x) * ease
      pointer.y += (pointer.ty - pointer.y) * ease
      rig.rotation.y = pointer.x * 0.45 + Math.sin(t * 0.25) * 0.08
      rig.rotation.x = pointer.y * 0.3
      renderer.render(scene, camera)
    }

    let running = false
    let raf = 0
    let last = performance.now()
    let t = 2
    const loop = (now: number) => {
      const dt = Math.min((now - last) / 1000, 1 / 30)
      last = now
      t += dt
      frame(t, dt)
      raf = requestAnimationFrame(loop)
    }
    const start = () => {
      if (running || reduced) return
      running = true
      last = performance.now()
      raf = requestAnimationFrame(loop)
    }
    const stop = () => {
      running = false
      cancelAnimationFrame(raf)
    }
    redrawRef.current = () => {
      if (!running) frame(t, 0)
    }
    themeRef.current = (isLight) => {
      applyTheme(isLight)
      if (!running) frame(t, 0)
    }

    let onScreen = true
    const io = new IntersectionObserver(([entry]) => {
      onScreen = Boolean(entry?.isIntersecting)
      if (onScreen && !document.hidden) start()
      else stop()
    })
    io.observe(container)
    const onVisibility = () => (document.hidden || !onScreen ? stop() : start())
    document.addEventListener("visibilitychange", onVisibility)
    const resizeObserver = new ResizeObserver(() => {
      resize()
      if (!running) frame(t, 0)
    })
    resizeObserver.observe(container)
    resize()
    frame(t, 0)
    if (!reduced) {
      window.addEventListener("pointermove", onPointer, { passive: true })
      start()
    }

    return () => {
      stop()
      redrawRef.current = null
      themeRef.current = null
      io.disconnect()
      resizeObserver.disconnect()
      document.removeEventListener("visibilitychange", onVisibility)
      window.removeEventListener("pointermove", onPointer)
      geometry.dispose()
      material.dispose()
      renderer.dispose()
      renderer.domElement.remove()
    }
  }, [])

  return <div ref={containerRef} aria-hidden className="pointer-events-none absolute inset-0 [&>canvas]:block [&>canvas]:h-full [&>canvas]:w-full" />
}
