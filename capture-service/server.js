/**
 * rsudp Capture Service
 *
 * HTTP server that uses Playwright to take screenshots of the WebUI capture page.
 * Maintains a persistent Chromium browser instance and processes capture requests
 * sequentially with a bounded queue.
 *
 * Environment variables:
 *   CAPTURE_PORT  - HTTP server port (default: 9100)
 *   WEBUI_URL     - Base URL of the WebUI (default: http://localhost:3000)
 */

const http = require("http");
const { chromium } = require("playwright-core");

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const CAPTURE_PORT = parseInt(process.env.CAPTURE_PORT || "9100", 10);
const WEBUI_URL = (process.env.WEBUI_URL || "http://localhost:3000").replace(
  /\/$/,
  ""
);
const MAX_QUEUE_DEPTH = 3;
const CAPTURE_TIMEOUT_MS = 30_000;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

let browser = null;
let capturesCompleted = 0;
const startTime = Date.now();

// Simple sequential queue backed by a promise chain.
let pendingCount = 0;
let queueChain = Promise.resolve();

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

function log(level, message) {
  const ts = new Date().toISOString();
  console.log(`[${ts}] [${level}] ${message}`);
}

// ---------------------------------------------------------------------------
// Browser lifecycle
// ---------------------------------------------------------------------------

async function launchBrowser() {
  log("INFO", "Launching Chromium browser...");
  browser = await chromium.launch({
    headless: true,
    args: [
      "--no-sandbox",
      "--disable-setuid-sandbox",
      "--disable-dev-shm-usage",
      "--disable-gpu",
    ],
  });

  browser.on("disconnected", () => {
    log("WARN", "Browser disconnected unexpectedly");
    browser = null;
  });

  log("INFO", "Browser launched successfully");
}

async function closeBrowser() {
  if (browser) {
    try {
      await browser.close();
    } catch (_) {
      // ignore errors during shutdown
    }
    browser = null;
  }
}

// ---------------------------------------------------------------------------
// Capture logic
// ---------------------------------------------------------------------------

/**
 * Perform a single screenshot capture.
 *
 * Opens a new page, navigates to the capture URL, waits for the page to
 * signal readiness via `data-capture-ready` attribute on <body>, takes a
 * screenshot, closes the page, and returns the PNG buffer.
 */
async function performCapture(request) {
  if (!browser || !browser.isConnected()) {
    log("WARN", "Browser not connected, attempting relaunch...");
    await launchBrowser();
  }

  const {
    channels = [],
    start_time,
    end_time,
    intensity_class,
    intensity_value,
    backend_url,
    width = 1000,
    height,
  } = request;

  const viewportHeight = height || 500 * Math.max(channels.length, 1);

  // Build capture URL
  const params = new URLSearchParams();
  if (channels.length > 0) params.set("channels", channels.join(","));
  if (start_time) params.set("start", start_time);
  if (end_time) params.set("end", end_time);
  if (intensity_class !== undefined && intensity_class !== null)
    params.set("intensity_class", String(intensity_class));
  if (intensity_value !== undefined && intensity_value !== null)
    params.set("intensity_value", String(intensity_value));
  if (backend_url) params.set("backend_url", backend_url);

  const captureUrl = `${WEBUI_URL}/capture?${params.toString()}`;

  log("INFO", `Capture start: ${captureUrl} (${width}x${viewportHeight})`);

  let page = null;
  try {
    page = await browser.newPage();
    await page.setViewportSize({ width, height: viewportHeight });

    await page.goto(captureUrl, {
      waitUntil: "domcontentloaded",
      timeout: CAPTURE_TIMEOUT_MS,
    });

    // Wait for the page to signal that rendering is complete.
    await page.waitForFunction(
      () => document.body.dataset.captureReady === "true",
      { timeout: CAPTURE_TIMEOUT_MS }
    );

    const screenshotBuffer = await page.screenshot({
      type: "png",
      fullPage: false,
    });

    capturesCompleted++;
    log("INFO", `Capture complete (#${capturesCompleted})`);

    return screenshotBuffer;
  } finally {
    if (page) {
      try {
        await page.close();
      } catch (_) {
        // ignore
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Queue management
// ---------------------------------------------------------------------------

/**
 * Enqueue a capture request. Returns a promise that resolves with the PNG
 * buffer or rejects on error. Rejects immediately with a 503-style error
 * if the queue depth exceeds MAX_QUEUE_DEPTH.
 */
function enqueueCapture(request) {
  if (pendingCount > MAX_QUEUE_DEPTH) {
    const err = new Error("Capture queue full");
    err.statusCode = 503;
    return Promise.reject(err);
  }

  pendingCount++;

  return new Promise((resolve, reject) => {
    queueChain = queueChain
      .then(() => performCapture(request))
      .then((buf) => {
        pendingCount--;
        resolve(buf);
      })
      .catch((err) => {
        pendingCount--;
        reject(err);
      });
  });
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on("data", (chunk) => chunks.push(chunk));
    req.on("end", () => resolve(Buffer.concat(chunks).toString("utf-8")));
    req.on("error", reject);
  });
}

function sendJson(res, statusCode, obj) {
  const body = JSON.stringify(obj);
  res.writeHead(statusCode, {
    "Content-Type": "application/json",
    "Content-Length": Buffer.byteLength(body),
  });
  res.end(body);
}

function sendPng(res, buffer) {
  res.writeHead(200, {
    "Content-Type": "image/png",
    "Content-Length": buffer.length,
  });
  res.end(buffer);
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

async function handleCapture(req, res) {
  if (req.method !== "POST") {
    sendJson(res, 405, { error: "Method not allowed" });
    return;
  }

  let body;
  try {
    const raw = await readBody(req);
    body = JSON.parse(raw);
  } catch (_) {
    sendJson(res, 400, { error: "Invalid JSON body" });
    return;
  }

  try {
    const pngBuffer = await enqueueCapture(body);
    sendPng(res, pngBuffer);
  } catch (err) {
    if (err.statusCode === 503) {
      log("WARN", "Queue full, rejecting request");
      sendJson(res, 503, { error: "Capture queue full, try again later" });
    } else {
      log("ERROR", `Capture failed: ${err.message}`);
      sendJson(res, 500, { error: `Capture failed: ${err.message}` });
    }
  }
}

function handleHealth(_req, res) {
  const uptimeSeconds = Math.floor((Date.now() - startTime) / 1000);
  sendJson(res, 200, {
    status: "ready",
    browser_connected: browser !== null && browser.isConnected(),
    uptime_seconds: uptimeSeconds,
    captures_completed: capturesCompleted,
  });
}

// ---------------------------------------------------------------------------
// Server
// ---------------------------------------------------------------------------

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://localhost:${CAPTURE_PORT}`);
  const pathname = url.pathname;

  if (pathname === "/capture") {
    await handleCapture(req, res);
  } else if (pathname === "/health") {
    handleHealth(req, res);
  } else {
    sendJson(res, 404, { error: "Not found" });
  }
});

// ---------------------------------------------------------------------------
// Startup & graceful shutdown
// ---------------------------------------------------------------------------

async function start() {
  await launchBrowser();

  server.listen(CAPTURE_PORT, () => {
    log("INFO", `Capture service listening on port ${CAPTURE_PORT}`);
    log("INFO", `WebUI URL: ${WEBUI_URL}`);
  });
}

async function shutdown(signal) {
  log("INFO", `Received ${signal}, shutting down...`);

  server.close(() => {
    log("INFO", "HTTP server closed");
  });

  await closeBrowser();
  log("INFO", "Browser closed");

  process.exit(0);
}

process.on("SIGTERM", () => shutdown("SIGTERM"));
process.on("SIGINT", () => shutdown("SIGINT"));

start().catch((err) => {
  log("ERROR", `Failed to start: ${err.message}`);
  process.exit(1);
});
