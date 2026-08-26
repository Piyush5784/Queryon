"use client"

import { useEffect, useRef } from "react"
import * as THREE from "three"
import { EffectComposer } from "three/addons/postprocessing/EffectComposer.js"
import { RenderPass } from "three/addons/postprocessing/RenderPass.js"
import { UnrealBloomPass } from "three/addons/postprocessing/UnrealBloomPass.js"
import { OutputPass } from "three/addons/postprocessing/OutputPass.js"

const NODE_COUNT = 85
const CONNECT_DISTANCE = 4.0
const FIELD_RADIUS = 6

function isWebGLAvailable() {
  try {
    const canvas = document.createElement("canvas")
    return !!(
      window.WebGLRenderingContext &&
      (canvas.getContext("webgl2") || canvas.getContext("webgl"))
    )
  } catch {
    return false
  }
}

export function HeroScene() {
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const container = containerRef.current
    if (!container) return
    if (!isWebGLAvailable()) return

    const isDark = document.documentElement.classList.contains("dark")
    const nodeColor = isDark ? 0x9cc0ff : 0x3b6fe0
    const lineColor = isDark ? 0x4a5d8a : 0xaebfe0

    const scene = new THREE.Scene()
    const camera = new THREE.PerspectiveCamera(70, 1, 0.1, 100)
    camera.position.set(0, 0, 7)

    let renderer: THREE.WebGLRenderer
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true })
    } catch {
      return
    }
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
    container.appendChild(renderer.domElement)

    scene.add(new THREE.AmbientLight(0xffffff, 0.5))
    const keyLight = new THREE.PointLight(isDark ? 0x8ab4ff : 0xffffff, 6, 30)
    keyLight.position.set(4, 3, 6)
    scene.add(keyLight)

    const group = new THREE.Group()
    scene.add(group)

    const nodes: THREE.Vector3[] = []
    for (let i = 0; i < NODE_COUNT; i++) {
      const r = FIELD_RADIUS * Math.cbrt(Math.random())
      const theta = Math.random() * Math.PI * 2
      const phi = Math.acos(2 * Math.random() - 1)
      nodes.push(
        new THREE.Vector3(
          r * Math.sin(phi) * Math.cos(theta) * 2.6,
          r * Math.sin(phi) * Math.sin(theta) * 0.9,
          r * Math.cos(phi) * 0.6
        )
      )
    }

    const sphereGeo = new THREE.SphereGeometry(0.09, 16, 16)
    const sphereMat = new THREE.MeshStandardMaterial({
      color: nodeColor,
      emissive: nodeColor,
      emissiveIntensity: 1.15,
      roughness: 0.35,
      metalness: 0.2,
    })
    nodes.forEach((pos) => {
      const mesh = new THREE.Mesh(sphereGeo, sphereMat)
      mesh.position.copy(pos)
      group.add(mesh)
    })

    const linePositions: number[] = []
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        if (nodes[i]!.distanceTo(nodes[j]!) < CONNECT_DISTANCE) {
          linePositions.push(nodes[i]!.x, nodes[i]!.y, nodes[i]!.z)
          linePositions.push(nodes[j]!.x, nodes[j]!.y, nodes[j]!.z)
        }
      }
    }
    const lineGeo = new THREE.BufferGeometry()
    lineGeo.setAttribute("position", new THREE.Float32BufferAttribute(linePositions, 3))
    const lineMat = new THREE.LineBasicMaterial({
      color: lineColor,
      transparent: true,
      opacity: 0.5,
      blending: THREE.AdditiveBlending,
    })
    group.add(new THREE.LineSegments(lineGeo, lineMat))

    const composer = new EffectComposer(renderer)
    composer.addPass(new RenderPass(scene, camera))
    const bloomPass = new UnrealBloomPass(new THREE.Vector2(1, 1), 0.8, 0.65, 0.18)
    composer.addPass(bloomPass)
    composer.addPass(new OutputPass())

    let raf = 0
    const clock = new THREE.Clock()

    const resize = () => {
      const { clientWidth, clientHeight } = container
      if (clientWidth === 0 || clientHeight === 0) return
      camera.aspect = clientWidth / clientHeight
      camera.updateProjectionMatrix()
      renderer.setSize(clientWidth, clientHeight)
      composer.setSize(clientWidth, clientHeight)
      bloomPass.setSize(clientWidth, clientHeight)
    }
    resize()

    const resizeObserver = new ResizeObserver(resize)
    resizeObserver.observe(container)

    const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches

    const animate = () => {
      raf = requestAnimationFrame(animate)
      const elapsed = clock.getElapsedTime()
      if (!prefersReducedMotion) {
        group.rotation.y = elapsed * 0.05
        group.rotation.x = Math.sin(elapsed * 0.15) * 0.06
      }
      composer.render()
    }
    animate()

    return () => {
      cancelAnimationFrame(raf)
      resizeObserver.disconnect()
      sphereGeo.dispose()
      sphereMat.dispose()
      lineGeo.dispose()
      lineMat.dispose()
      composer.dispose()
      renderer.dispose()
      container.removeChild(renderer.domElement)
    }
  }, [])

  return (
    <div
      ref={containerRef}
      aria-hidden
      className="pointer-events-none absolute inset-0 [&>canvas]:h-full [&>canvas]:w-full"
    />
  )
}
