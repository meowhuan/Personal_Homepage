#!/usr/bin/env node

import { chromium } from "playwright";

function argValue(name, fallback = "") {
  const idx = process.argv.indexOf(name);
  if (idx === -1 || idx + 1 >= process.argv.length) return fallback;
  return process.argv[idx + 1];
}

async function main() {
  const targetUrl = argValue("--url");
  const waitUntil = argValue("--wait-until", "networkidle");
  const waitAfterMs = Number(argValue("--wait-after-ms", "800")) || 0;

  if (!targetUrl) {
    console.error("missing --url");
    process.exit(2);
  }

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    userAgent:
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
  });
  const page = await context.newPage();
  try {
    await page.goto(targetUrl, {
      waitUntil: ["load", "domcontentloaded", "networkidle"].includes(waitUntil)
        ? waitUntil
        : "networkidle",
      timeout: 15000,
    });
    if (waitAfterMs > 0) {
      await page.waitForTimeout(Math.min(waitAfterMs, 5000));
    }
    const html = await page.content();
    process.stdout.write(html || "");
  } finally {
    await context.close();
    await browser.close();
  }
}

main().catch((err) => {
  console.error(err?.message || String(err));
  process.exit(1);
});
