/**
 * Demo GIF recorder for ALIV README.
 *
 * Records the Query screen with a live retrieval response and exports
 * a looping GIF suitable for embedding in the GitHub README.
 *
 * Prerequisites:
 *   npm install puppeteer gif-encoder-2 canvas
 *
 * Usage (while `npm run dev` is running in another terminal):
 *   node scripts/record-demo-gif.mjs
 *
 * Output: docs/assets/demo.gif  (~2–4 MB, 1200×720, 15 fps, 20 seconds)
 */

import puppeteer from 'puppeteer'
import GIFEncoder from 'gif-encoder-2'
import { createCanvas, loadImage } from 'canvas'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const OUT_DIR = path.join(__dirname, '..', 'docs', 'assets')
const OUT_FILE = path.join(OUT_DIR, 'demo.gif')

const WIDTH = 1200
const HEIGHT = 720
const FPS = 15
const FRAME_DELAY = Math.round(1000 / FPS)

// Sequence: each step is [wait_ms, optional_action]
// Tweak timings to match your machine's LLM response speed
const SEQUENCE = [
  // Phase 1 — Land on Query screen
  { wait: 1200 },
  { navigate: 'http://localhost:1420/#/query', wait: 800 },

  // Phase 2 — Type the query
  { type: '[placeholder="What lease clauses mention unilateral renewal rights"]', text: 'Find all agreements expiring in 90 days and extract the penalty clauses.', wait: 1400 },

  // Phase 3 — Run the query (Cmd+Enter shortcut)
  { shortcut: 'Meta+Enter', wait: 6000 },  // wait for LLM response

  // Phase 4 — Scroll down to show evidence cards
  { scroll: 400, wait: 1200 },
  { wait: 3000 }, // hold on results

  // Phase 5 — Navigate to Actions Preview
  { navigate: 'http://localhost:1420/#/actions', wait: 800 },
  { click: 'button:has-text("Generate preview")', wait: 2000 },
  { wait: 2500 }, // hold on preview table

  // Phase 6 — Loop back to start
  { navigate: 'http://localhost:1420/#/query', wait: 1000 },
]

async function captureFrames(page) {
  const frames = []

  for (const step of SEQUENCE) {
    if (step.navigate) {
      await page.goto(step.navigate, { waitUntil: 'networkidle2' })
    }
    if (step.type) {
      await page.click(step.type)
      await page.keyboard.down('Control')
      await page.keyboard.press('a')
      await page.keyboard.up('Control')
      await page.keyboard.type(step.text, { delay: 30 })
    }
    if (step.shortcut) {
      const keys = step.shortcut.split('+')
      for (const k of keys.slice(0, -1)) await page.keyboard.down(k)
      await page.keyboard.press(keys[keys.length - 1])
      for (const k of keys.slice(0, -1)) await page.keyboard.up(k)
    }
    if (step.click) {
      try {
        await page.click(step.click)
      } catch { /* element not found — skip */ }
    }
    if (step.scroll) {
      await page.evaluate((px) => window.scrollBy(0, px), step.scroll)
    }

    const waitMs = step.wait ?? 500
    const frameCount = Math.max(1, Math.round(waitMs / FRAME_DELAY))
    for (let i = 0; i < frameCount; i++) {
      await new Promise((r) => setTimeout(r, FRAME_DELAY))
      const buf = await page.screenshot({ type: 'png', clip: { x: 0, y: 0, width: WIDTH, height: HEIGHT } })
      frames.push(buf)
    }
  }

  return frames
}

async function encodeGif(frames) {
  fs.mkdirSync(OUT_DIR, { recursive: true })

  const encoder = new GIFEncoder(WIDTH, HEIGHT, 'neuquant', true)
  const stream = encoder.createReadStream()
  const chunks = []
  stream.on('data', (c) => chunks.push(c))

  encoder.setDelay(FRAME_DELAY)
  encoder.setQuality(12)   // 1 = best, 20 = fastest
  encoder.setRepeat(0)     // 0 = loop forever
  encoder.start()

  const canvas = createCanvas(WIDTH, HEIGHT)
  const ctx = canvas.getContext('2d')

  console.log(`Encoding ${frames.length} frames…`)
  for (let i = 0; i < frames.length; i++) {
    if (i % 30 === 0) process.stdout.write(`  frame ${i}/${frames.length}\r`)
    const img = await loadImage(frames[i])
    ctx.clearRect(0, 0, WIDTH, HEIGHT)
    ctx.drawImage(img, 0, 0)
    encoder.addFrame(ctx)
  }

  encoder.finish()
  await new Promise((r) => stream.on('end', r))

  const buf = Buffer.concat(chunks)
  fs.writeFileSync(OUT_FILE, buf)
  console.log(`\nGIF written: ${OUT_FILE}  (${(buf.length / 1024 / 1024).toFixed(1)} MB, ${frames.length} frames)`)
}

async function main() {
  console.log('Launching browser…')
  const browser = await puppeteer.launch({
    headless: 'new',
    defaultViewport: { width: WIDTH, height: HEIGHT },
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  })

  const page = await browser.newPage()

  // Wait for Vite dev server
  let ready = false
  for (let attempt = 0; attempt < 20; attempt++) {
    try {
      await page.goto('http://localhost:1420', { waitUntil: 'networkidle2', timeout: 5000 })
      ready = true
      break
    } catch {
      console.log(`Waiting for dev server… (attempt ${attempt + 1}/20)`)
      await new Promise((r) => setTimeout(r, 1500))
    }
  }
  if (!ready) throw new Error('Dev server not reachable at http://localhost:1420 — run `npm run dev` first.')

  console.log('Capturing frames…')
  const frames = await captureFrames(page)
  await browser.close()

  await encodeGif(frames)
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
