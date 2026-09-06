// Rendered responsive gate for the Virya Signal WebView.
//
// The stylesheet has 28 media queries and nothing ever proved they worked. CSS
// inspection cannot: the failures that matter come from what the *content* does
// to the layout, not from what the rules say. This drives the real built bundle
// in a real browser through the preview harness, at the widths real phones use,
// and measures the boxes.
//
// The load-bearing check is not `document.scrollWidth`. The shell is
// `height: 100dvh; overflow: hidden`, so the document never scrolls and
// scrollWidth stays equal to the viewport even while content sits a thousand
// pixels off-screen. Overflow here is silent clipping, which is worse than a
// scrollbar because nothing hints that anything is missing. So every element is
// measured against the viewport, and an element is only excused when an
// ancestor actually scrolls horizontally.
//
// The stress pass is the point. Display names, emails, referral URLs and event
// titles all come from CrowdRelay, none of them is guaranteed to contain a
// space, and a word with no break opportunity is laid out at full width however
// narrow its box is. Without that pass this gate would report a clean sweep on
// content that happens to wrap.
//
// Usage: node scripts/check-responsive.mjs [--base http://127.0.0.1:4181]
//
// The repository has no package.json, so there is no node_modules beside this
// file for ESM to resolve a bare `playwright` specifier against. VIRYA_PLAYWRIGHT
// names the installed module explicitly instead of pretending a tree exists:
//   npm --prefix "$RUNNER_TEMP/rwd" install playwright
//   VIRYA_PLAYWRIGHT="$RUNNER_TEMP/rwd/node_modules/playwright/index.mjs" node scripts/check-responsive.mjs
const { chromium } = await import(process.env.VIRYA_PLAYWRIGHT ?? 'playwright');

const BASE = (() => {
  const i = process.argv.indexOf('--base');
  return i !== -1 ? process.argv[i + 1] : 'http://127.0.0.1:4181';
})();

const WIDTHS = [320, 360, 375, 390, 430, 768, 1024, 1280, 1440];

// Each mode is a distinct authenticated surface in the preview harness.
// `link=1` stages a pending mailed confirmation link, which is its own screen.
const MODES = [
  { id: 'fan-out', label: 'signup' },
  { id: 'fan-locked', label: 'locked gate' },
  { id: 'fan-locked&link=1', label: 'pending confirmation link' },
  { id: 'fan', label: 'fan shell', tabs: true },
  // `push=prompt` is the only state the notification primer renders in, and
  // a centered dialog is exactly the thing a transformed ancestor pushes
  // off-screen without any scrollbar to show for it.
  { id: 'fan&push=prompt', label: 'notification primer' },
  { id: 'beacon', label: 'beacon' },
  { id: 'staff', label: 'staff gate' },
  { id: 'owner', label: 'operator shell', tabs: true },
];

const TOUCH_MIN = 44;
const INPUT_FONT_MIN = 16;

const MEASURE = ({ touchMin, inputFontMin }) => {
  const viewport = document.documentElement.clientWidth;
  const describe = (el) => ({
    tag: el.tagName.toLowerCase(),
    cls: (el.className && el.className.toString ? el.className.toString() : '').slice(0, 60),
    text: (el.textContent || '').trim().slice(0, 40),
  });

  const scrolls = (el) => {
    let node = el.parentElement;
    while (node) {
      if (/auto|scroll/.test(getComputedStyle(node).overflowX)) return true;
      node = node.parentElement;
    }
    return false;
  };

  const overflow = [];
  for (const el of document.querySelectorAll('*')) {
    const rect = el.getBoundingClientRect();
    if (rect.width === 0 && rect.height === 0) continue;
    // One pixel of tolerance for sub-pixel layout rounding.
    if (rect.right <= viewport + 1 && rect.left >= -1) continue;
    if (scrolls(el)) continue;
    overflow.push({ ...describe(el), left: Math.round(rect.left), right: Math.round(rect.right) });
  }

  // The tappable box is whatever the finger can land on, so a small control
  // inside a large label is not a failure — the label is the target.
  const small = [];
  for (const el of document.querySelectorAll('button, a[href], [role="button"], input[type="checkbox"], input[type="radio"]')) {
    const rect = el.getBoundingClientRect();
    if (rect.width === 0 && rect.height === 0) continue;
    const label = el.closest('label');
    const box = label ? label.getBoundingClientRect() : rect;
    if (box.width >= touchMin && box.height >= touchMin) continue;
    small.push({ ...describe(el), width: Math.round(box.width), height: Math.round(box.height) });
  }

  // Anything under 16px makes iOS Safari zoom the page on focus, which is a
  // layout break the user has to undo by hand.
  const tinyText = [];
  for (const el of document.querySelectorAll('input, select, textarea')) {
    const rect = el.getBoundingClientRect();
    if (rect.width === 0 && rect.height === 0) continue;
    const size = parseFloat(getComputedStyle(el).fontSize);
    if (size >= inputFontMin) continue;
    tinyText.push({ ...describe(el), fontSize: size });
  }

  return { viewport, scrollWidth: document.documentElement.scrollWidth, overflow, small, tinyText };
};

// Surfaces that render a string the client did not choose — a display name, an
// email, an event title, a coupon code, a referral URL. These are the only ones
// worth stressing: button labels and headings come from the i18n catalogs,
// which are ours, so a 72-character word there is not a threat model, it is a
// typo we would catch in review.
//
// Add a selector here whenever a component starts rendering server data.
const SERVER_TEXT_SELECTORS = [
  '.topbar strong',
  '.hero-card h3',
  '.hero-card p',
  '.fan-event-card strong',
  '.fan-event-card p',
  '.fan-event-detail strong',
  '.fan-event-detail p',
  '.event-description',
  '.draw-card strong',
  '.fan-coupon strong',
  '.reward-card strong',
  '.reward-card p',
  '.referral-code-copy',
  '.admission-card strong',
  '.settings-row small',
  '.settings-row strong',
  '.empty-state strong',
];

// Values shaped like the ones CrowdRelay can actually return: no break
// opportunity anywhere in them.
const STRESS = (selectors) => {
  const long = 'a'.repeat(72);
  const values = [
    `https://virya.music/pl/dowody/losowania/${long}`,
    `bardzo.dluga.nazwa.uzytkownika.${long}@example.com`,
    long,
  ];
  let applied = 0;
  for (const selector of selectors) {
    for (const el of document.querySelectorAll(selector)) {
      const rect = el.getBoundingClientRect();
      if (rect.width === 0 && rect.height === 0) continue;
      el.textContent = values[applied % values.length];
      applied += 1;
    }
  }
  return applied;
};

const failures = [];

const record = (where, result) => {
  for (const item of result.overflow) {
    failures.push(
      `${where}: <${item.tag} class="${item.cls}"> spans ${item.left}..${item.right} ` +
        `past a ${result.viewport}px viewport — "${item.text}"`
    );
  }
  for (const item of result.small) {
    failures.push(
      `${where}: <${item.tag} class="${item.cls}"> tap target ${item.width}x${item.height} ` +
        `is under ${TOUCH_MIN}x${TOUCH_MIN} — "${item.text}"`
    );
  }
  for (const item of result.tinyText) {
    failures.push(
      `${where}: <${item.tag} class="${item.cls}"> font-size ${item.fontSize}px is under ` +
        `${INPUT_FONT_MIN}px and will zoom iOS Safari on focus`
    );
  }
};

const browser = await chromium.launch();
let checks = 0;

try {
  for (const mode of MODES) {
    for (const width of WIDTHS) {
      const context = await browser.newContext({
        viewport: { width, height: width < 768 ? 780 : 900 },
        deviceScaleFactor: 2,
      });
      const page = await context.newPage();
      await page.goto(`${BASE}/index.html?mode=${mode.id}`, { waitUntil: 'networkidle' });
      await page.waitForSelector('.app-shell', { timeout: 20000 });
      // The shell paints before its sections settle; give the reactive owners a
      // beat so the measurement is of the populated screen, not the skeleton.
      await page.waitForTimeout(1200);

      const options = { touchMin: TOUCH_MIN, inputFontMin: INPUT_FONT_MIN };
      record(`${mode.label} @${width}`, await page.evaluate(MEASURE, options));
      checks += 1;

      if (mode.tabs) {
        const count = await page.locator('.bottom-nav button').count();
        for (let index = 1; index < count; index += 1) {
          await page.locator('.bottom-nav button').nth(index).click();
          await page.waitForTimeout(900);
          const name = (await page.locator('.bottom-nav button').nth(index).innerText()).trim();
          record(`${mode.label} tab ${name} @${width}`, await page.evaluate(MEASURE, options));
          checks += 1;
        }
      }

      const applied = await page.evaluate(STRESS, SERVER_TEXT_SELECTORS);
      if (applied > 0) {
        await page.waitForTimeout(500);
        record(`${mode.label} long-text @${width}`, await page.evaluate(MEASURE, options));
        checks += 1;
      }

      await context.close();
    }
  }
} finally {
  await browser.close();
}

if (failures.length > 0) {
  const shown = failures.slice(0, 40);
  for (const failure of shown) console.error(`- ${failure}`);
  if (failures.length > shown.length) {
    console.error(`- ... and ${failures.length - shown.length} more`);
  }
  console.error(`SIGNAL_RESPONSIVE=FAIL checks=${checks} failures=${failures.length}`);
  process.exit(1);
}

console.log(
  `SIGNAL_RESPONSIVE=PASS checks=${checks} widths=${WIDTHS.length} surfaces=${MODES.length}`
);
