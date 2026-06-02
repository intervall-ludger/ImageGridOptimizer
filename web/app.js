import init, { solve_collage } from './pkg/collage_core.js';

const $ = (id) => document.getElementById(id);
const els = {
  files: $('files'), drop: $('drop'), dropLabel: $('dropLabel'),
  flex: $('flex'), aspect: $('aspect'), min: $('min'), max: $('max'), rotate: $('rotate'),
  gutter: $('gutter'), margin: $('margin'), width: $('width'), pop: $('pop'), gens: $('gens'),
  generate: $('generate'), download: $('download'), status: $('status'), canvas: $('canvas'),
};

let images = [];
let nextId = 0;
let lastLayout = null;
let ready = false;

await init();
ready = true;

// Live value readouts next to each slider.
const outs = { flex: 'flexOut', aspect: 'aspectOut', min: 'minOut', max: 'maxOut',
  gutter: 'gutterOut', margin: 'marginOut', width: 'widthOut', pop: 'popOut', gens: 'gensOut' };
for (const [key, outId] of Object.entries(outs)) {
  const el = els[key], out = $(outId);
  const sync = () => { out.textContent = key === 'flex' || key === 'aspect' ? (+el.value).toFixed(2) : el.value; };
  el.addEventListener('input', sync);
  el.addEventListener('change', () => { if (images.length) generate(); });
  sync();
}

els.files.addEventListener('change', (e) => addFiles(e.target.files));
els.generate.addEventListener('click', generate);
els.download.addEventListener('click', downloadCollage);

['dragover', 'dragenter'].forEach((ev) =>
  els.drop.addEventListener(ev, (e) => { e.preventDefault(); els.drop.classList.add('dragover'); }));
['dragleave', 'drop'].forEach((ev) =>
  els.drop.addEventListener(ev, (e) => { e.preventDefault(); els.drop.classList.remove('dragover'); }));
els.drop.addEventListener('drop', (e) => addFiles(e.dataTransfer.files));

async function addFiles(fileList) {
  const files = [...fileList].filter((f) => f.type.startsWith('image/'));
  if (!files.length) return;
  els.status.textContent = `Loading ${files.length} image(s)…`;
  for (const file of files) {
    try {
      const img = await loadImage(file);
      images.push({ id: nextId++, img, w: img.naturalWidth, h: img.naturalHeight });
    } catch {
      // skip files that fail to decode
    }
  }
  els.dropLabel.textContent = `${images.length} image(s) loaded — click or drop to add more`;
  els.generate.disabled = images.length === 0;
  if (images.length) generate();
}

function loadImage(file) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = URL.createObjectURL(file);
  });
}

function params() {
  const lo = Math.min(+els.min.value, +els.max.value);
  const hi = Math.max(+els.min.value, +els.max.value);
  return {
    target_aspect: +els.aspect.value,
    flex: +els.flex.value,
    min_images: lo,
    max_images: hi,
    allow_rotate: els.rotate.checked,
    population_size: +els.pop.value,
    generations: +els.gens.value,
    mutation_rate: 0.3,
    width: +els.width.value,
  };
}

function generate() {
  if (!ready || !images.length) return;
  els.status.textContent = 'Packing…';
  // Defer one frame so the status text paints before the synchronous solve.
  requestAnimationFrame(() => {
    const imagesJson = JSON.stringify(images.map((i) => ({ id: i.id, w: i.w, h: i.h })));
    const t0 = performance.now();
    let layout;
    try {
      layout = JSON.parse(solve_collage(imagesJson, JSON.stringify(params())));
    } catch (e) {
      els.status.textContent = `Error: ${e.message ?? e}`;
      return;
    }
    render(layout);
    lastLayout = layout;
    els.download.disabled = false;
    els.status.textContent = `${layout.cells.length} images · ${Math.round(performance.now() - t0)} ms`;
  });
}

function render(layout) {
  const gutter = +els.gutter.value;
  const margin = +els.margin.value;
  const cw = layout.width + 2 * margin;
  const ch = layout.height + 2 * margin;
  els.canvas.width = cw;
  els.canvas.height = ch;
  const ctx = els.canvas.getContext('2d');
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, cw, ch);

  const byId = new Map(images.map((i) => [i.id, i.img]));
  for (const c of layout.cells) {
    const img = byId.get(c.id);
    if (!img) continue;
    // On-screen source size (rotated swaps width/height).
    const sw = c.rotated ? img.naturalHeight : img.naturalWidth;
    const sh = c.rotated ? img.naturalWidth : img.naturalHeight;
    const availW = Math.max(1, c.w - gutter);
    const availH = Math.max(1, c.h - gutter);
    const scale = Math.min(availW / sw, availH / sh);
    const drawW = Math.max(1, Math.round(sw * scale));
    const drawH = Math.max(1, Math.round(sh * scale));
    const tx = margin + c.x + Math.floor((c.w - drawW) / 2);
    const ty = margin + c.y + Math.floor((c.h - drawH) / 2);

    if (c.rotated) {
      ctx.save();
      ctx.translate(tx + drawW / 2, ty + drawH / 2);
      ctx.rotate(Math.PI / 2);
      ctx.drawImage(img, -drawH / 2, -drawW / 2, drawH, drawW);
      ctx.restore();
    } else {
      ctx.drawImage(img, tx, ty, drawW, drawH);
    }
  }
}

function downloadCollage() {
  els.canvas.toBlob((blob) => {
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = 'collage.png';
    a.click();
    URL.revokeObjectURL(a.href);
  }, 'image/png');
}
