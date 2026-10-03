"use client";

import { useEffect, useRef } from "react";
import * as THREE from "three";

const DISKS = 3;
const DISK_HEIGHT = 0.82;
const DISK_GAP = 0.16;
const RADIUS = 1.55;
const ORBITS = [
  { radius: 2.55, tiltX: 1.18, tiltZ: 0.18, speed: 0.22, pulses: 4 },
  { radius: 2.95, tiltX: 1.32, tiltZ: -0.32, speed: -0.16, pulses: 3 },
  { radius: 3.35, tiltX: 1.05, tiltZ: 0.5, speed: 0.12, pulses: 3 },
];

const shellVertex = `
  attribute float aRow;
  attribute float aSeed;
  attribute float aRim;
  uniform float uTime;
  uniform float uScale;
  varying float vGlow;
  void main() {
    float scan = fract(uTime * 0.16);
    float band = exp(-pow((aRow - scan) * 9.0, 2.0));
    float twinkle = 0.5 + 0.5 * sin(uTime * 2.4 + aSeed * 60.0);
    vGlow = 0.22 + band * 0.9 + aRim * 0.55 + twinkle * 0.12;
    vec4 mv = modelViewMatrix * vec4(position, 1.0);
    gl_PointSize = uScale * (1.6 + aRim * 1.4 + band * 1.6) * (6.0 / -mv.z);
    gl_Position = projectionMatrix * mv;
  }
`;

const shellFragment = `
  varying float vGlow;
  void main() {
    float d = length(gl_PointCoord - 0.5);
    if (d > 0.5) discard;
    vec3 col = mix(vec3(0.13, 0.77, 0.37), vec3(0.78, 1.0, 0.86), clamp(vGlow - 0.7, 0.0, 1.0));
    gl_FragColor = vec4(col * vGlow, smoothstep(0.5, 0.05, d) * min(vGlow, 1.0));
  }
`;

export function isWebGLAvailable() {
  try {
    const canvas = document.createElement("canvas");
    return Boolean(
      window.WebGLRenderingContext &&
      (canvas.getContext("webgl2") || canvas.getContext("webgl")),
    );
  } catch {
    return false;
  }
}

export function HeroScene() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !isWebGLAvailable()) return;

    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({
        antialias: true,
        alpha: true,
        powerPreference: "high-performance",
      });
    } catch {
      return;
    }
    const reduced = window.matchMedia(
      "(prefers-reduced-motion: reduce)",
    ).matches;
    const dpr = Math.min(window.devicePixelRatio, 2);
    renderer.setPixelRatio(dpr);
    renderer.setClearColor(0x000000, 0);
    container.appendChild(renderer.domElement);

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(36, 1, 0.1, 100);
    camera.position.set(0, 1.6, 9.5);
    camera.lookAt(0, 0, 0);

    const rig = new THREE.Group();
    rig.rotation.x = 0.12;
    scene.add(rig);

    const positions: number[] = [];
    const rows: number[] = [];
    const seeds: number[] = [];
    const rims: number[] = [];
    const totalHeight = DISKS * DISK_HEIGHT + (DISKS - 1) * DISK_GAP;
    const around = 84;
    for (let d = 0; d < DISKS; d++) {
      const bottom = -totalHeight / 2 + d * (DISK_HEIGHT + DISK_GAP);
      const layers = 9;
      for (let l = 0; l <= layers; l++) {
        const y = bottom + (l / layers) * DISK_HEIGHT;
        const rim = l === 0 || l === layers ? 1 : 0;
        for (let a = 0; a < around; a++) {
          const angle =
            (a / around) * Math.PI * 2 + (l % 2) * (Math.PI / around);
          positions.push(Math.cos(angle) * RADIUS, y, Math.sin(angle) * RADIUS);
          rows.push((y + totalHeight / 2) / totalHeight);
          seeds.push(Math.random());
          rims.push(rim);
        }
      }
      if (d === DISKS - 1) {
        for (let ring = 1; ring < 7; ring++) {
          const r = (ring / 7) * RADIUS;
          const count = Math.round(around * (ring / 7));
          for (let a = 0; a < count; a++) {
            const angle = (a / count) * Math.PI * 2;
            positions.push(
              Math.cos(angle) * r,
              bottom + DISK_HEIGHT,
              Math.sin(angle) * r,
            );
            rows.push(1);
            seeds.push(Math.random());
            rims.push(0.35);
          }
        }
      }
    }
    const shellGeo = new THREE.BufferGeometry();
    shellGeo.setAttribute(
      "position",
      new THREE.Float32BufferAttribute(positions, 3),
    );
    shellGeo.setAttribute("aRow", new THREE.Float32BufferAttribute(rows, 1));
    shellGeo.setAttribute("aSeed", new THREE.Float32BufferAttribute(seeds, 1));
    shellGeo.setAttribute("aRim", new THREE.Float32BufferAttribute(rims, 1));
    const uniforms = { uTime: { value: 0 }, uScale: { value: 1 } };
    const shellMat = new THREE.ShaderMaterial({
      vertexShader: shellVertex,
      fragmentShader: shellFragment,
      uniforms,
      transparent: true,
      depthWrite: false,
      blending: THREE.AdditiveBlending,
    });
    rig.add(new THREE.Points(shellGeo, shellMat));

    const glowCanvas = document.createElement("canvas");
    glowCanvas.width = glowCanvas.height = 128;
    const g = glowCanvas.getContext("2d")!;
    const gradient = g.createRadialGradient(64, 64, 0, 64, 64, 64);
    gradient.addColorStop(0, "rgba(74,222,128,0.55)");
    gradient.addColorStop(0.4, "rgba(34,197,94,0.18)");
    gradient.addColorStop(1, "rgba(34,197,94,0)");
    g.fillStyle = gradient;
    g.fillRect(0, 0, 128, 128);
    const glowTex = new THREE.CanvasTexture(glowCanvas);
    const glowMat = new THREE.SpriteMaterial({
      map: glowTex,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
      transparent: true,
    });
    const glow = new THREE.Sprite(glowMat);
    glow.scale.set(7.5, 7.5, 1);
    rig.add(glow);

    const orbitMat = new THREE.LineBasicMaterial({
      color: 0x22c55e,
      transparent: true,
      opacity: 0.28,
      blending: THREE.AdditiveBlending,
    });
    const pulseGeo = new THREE.SphereGeometry(0.055, 12, 8);
    const pulseMat = new THREE.MeshBasicMaterial({ color: 0xc8ffd9 });
    const orbitGeos: THREE.BufferGeometry[] = [];
    const orbits = ORBITS.map((o) => {
      const group = new THREE.Group();
      group.rotation.set(o.tiltX, 0, o.tiltZ);
      const points: THREE.Vector3[] = [];
      for (let i = 0; i <= 160; i++) {
        const t = (i / 160) * Math.PI * 2;
        points.push(
          new THREE.Vector3(Math.cos(t) * o.radius, Math.sin(t) * o.radius, 0),
        );
      }
      const geo = new THREE.BufferGeometry().setFromPoints(points);
      orbitGeos.push(geo);
      group.add(new THREE.Line(geo, orbitMat));
      const pulses = Array.from({ length: o.pulses }, (_, i) => {
        const m = new THREE.Mesh(pulseGeo, pulseMat);
        group.add(m);
        return { mesh: m, offset: (i / o.pulses) * Math.PI * 2 };
      });
      rig.add(group);
      return { ...o, pulses };
    });

    const resize = () => {
      const { clientWidth: w, clientHeight: h } = container;
      if (!w || !h) return;
      renderer.setSize(w, h, false);
      camera.aspect = w / h;
      camera.position.z = camera.aspect < 0.9 ? 11.5 : 9.5;
      camera.updateProjectionMatrix();
      uniforms.uScale.value = (h * dpr) / 520;
    };

    const pointer = { x: 0, y: 0, tx: 0, ty: 0 };
    const onPointer = (event: PointerEvent) => {
      pointer.tx = event.clientX / window.innerWidth - 0.5;
      pointer.ty = event.clientY / window.innerHeight - 0.5;
    };

    const frame = (t: number, dt: number) => {
      uniforms.uTime.value = t;
      const ease = dt === 0 ? 1 : 1 - Math.pow(0.04, dt);
      pointer.x += (pointer.tx - pointer.x) * ease;
      pointer.y += (pointer.ty - pointer.y) * ease;
      rig.rotation.y = t * 0.18 + pointer.x * 0.5;
      rig.rotation.x = 0.12 + pointer.y * 0.25;
      orbits.forEach((o) => {
        o.pulses.forEach((p) => {
          const a = p.offset + t * o.speed * 3;
          p.mesh.position.set(
            Math.cos(a) * o.radius,
            Math.sin(a) * o.radius,
            0,
          );
        });
      });
      renderer.render(scene, camera);
    };

    let running = false;
    let raf = 0;
    let last = performance.now();
    let t = 0;
    const loop = (now: number) => {
      const dt = Math.min((now - last) / 1000, 1 / 30);
      last = now;
      t += dt;
      frame(t, dt);
      raf = requestAnimationFrame(loop);
    };
    const start = () => {
      if (running || reduced) return;
      running = true;
      last = performance.now();
      raf = requestAnimationFrame(loop);
    };
    const stop = () => {
      running = false;
      cancelAnimationFrame(raf);
    };

    let onScreen = true;
    const io = new IntersectionObserver(([entry]) => {
      onScreen = Boolean(entry?.isIntersecting);
      if (onScreen && !document.hidden) start();
      else stop();
    });
    io.observe(container);
    const onVisibility = () =>
      document.hidden || !onScreen ? stop() : start();
    document.addEventListener("visibilitychange", onVisibility);
    const resizeObserver = new ResizeObserver(() => {
      resize();
      if (!running) frame(t || 2, 0);
    });
    resizeObserver.observe(container);
    resize();
    frame(2, 0);
    if (!reduced) {
      window.addEventListener("pointermove", onPointer, { passive: true });
      start();
    }

    return () => {
      stop();
      io.disconnect();
      resizeObserver.disconnect();
      document.removeEventListener("visibilitychange", onVisibility);
      window.removeEventListener("pointermove", onPointer);
      shellGeo.dispose();
      shellMat.dispose();
      glowTex.dispose();
      glowMat.dispose();
      orbitMat.dispose();
      orbitGeos.forEach((geo) => geo.dispose());
      pulseGeo.dispose();
      pulseMat.dispose();
      renderer.dispose();
      renderer.domElement.remove();
    };
  }, []);

  return (
    <div
      ref={containerRef}
      aria-hidden
      className="pointer-events-none absolute inset-0 [&>canvas]:block [&>canvas]:h-full [&>canvas]:w-full"
    />
  );
}
