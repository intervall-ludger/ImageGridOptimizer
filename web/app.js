import init, { solve_collage } from './pkg/collage_core.js';

const $ = (id) => document.getElementById(id);
const els = {
  files: $('files'), drop: $('drop'), dropLabel: $('dropLabel'),
  flex: $('flex'), aspect: $('aspect'), min: $('min'), max: $('max'), rotate: $('rotate'),
  gutter: $('gutter'), margin: $('margin'), width: $('width'), pop: $('pop'), gens: $('gens'),
  download: $('download'), status: $('status'), canvas: $('canvas'),
  gallery: $('gallery'), galleryCount: $('galleryCount'),
};

let images = [];
let nextId = 0;
let ready = false;

await init();
ready = true;

// Live value readouts; any change re-runs the (instant) solve.
const outs = { flex: 'flexOut', aspect: 'aspectOut', min: 'minOut', max: 'maxOut',
  gutter: 'gutterOut', margin: 'marginOut', width: 'widthOut', pop: 'popOut', gens: 'gensOut' };
for (const [key, outId] of Object.entries(outs)) {
  const el = els[key], out = $(outId);
  const sync = () => { out.textContent = key === 'flex' || key === 'aspect' ? (+el.value).toFixed(2) : el.value; };
  el.addEventListener('input', sync);
  el.addEventListener('change', generate);
  sync();
}
els.rotate.addEventListener('change', generate);
els.files.addEventListener('change', (e) => addFiles(e.target.files));
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
      images.push({ id: nextId++, img, w: img.naturalWidth, h: img.naturalHeight, forced: false, unused: false });
    } catch {
      // skip files that fail to decode
    }
  }
  renderGallery();
  generate();
}

function loadImage(file) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = URL.createObjectURL(file);
  });
}

function removeImage(id) {
  const item = images.find((i) => i.id === id);
  if (item) URL.revokeObjectURL(item.img.src);
  images = images.filter((i) => i.id !== id);
  renderGallery();
  if (images.length) generate();
  else { clearCanvas(); els.download.disabled = true; els.status.textContent = 'Add some images to start.'; }
}

function toggleForce(id) {
  const item = images.find((i) => i.id === id);
  if (item) item.forced = !item.forced;
  renderGallery();
  generate();
}

function renderGallery() {
  els.gallery.innerHTML = '';
  els.galleryCount.textContent = images.length ? `${images.length}` : '';
  els.dropLabel.textContent = images.length
    ? 'Click or drop to add more'
    : 'Click or drop images here';
  for (const item of images) {
    const fig = document.createElement('div');
    fig.className = 'thumb' + (item.forced ? ' forced' : '') + (item.unused ? ' unused' : '');

    const im = document.createElement('img');
    im.src = item.img.src;
    fig.appendChild(im);

    const pin = document.createElement('button');
    pin.className = 'pin';
    pin.innerHTML = '<span class="icon">push_pin</span>';
    const pinLabel = item.forced ? 'Forced in — click to unpin' : 'Force this image into the collage';
    pin.title = pinLabel;
    pin.setAttribute('aria-label', pinLabel);
    pin.setAttribute('aria-pressed', String(item.forced));
    pin.addEventListener('click', () => toggleForce(item.id));
    fig.appendChild(pin);

    const del = document.createElement('button');
    del.className = 'del';
    del.innerHTML = '<span class="icon">close</span>';
    del.title = 'Remove image';
    del.setAttribute('aria-label', 'Remove image');
    del.addEventListener('click', () => removeImage(item.id));
    fig.appendChild(del);

    if (item.unused) {
      const badge = document.createElement('span');
      badge.className = 'badge';
      badge.textContent = 'unused';
      fig.appendChild(badge);
    }
    els.gallery.appendChild(fig);
  }
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
    forced: images.filter((i) => i.forced).map((i) => i.id),
  };
}

function generate() {
  if (!ready || !images.length) return;
  els.status.textContent = 'Packing…';
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
    const used = new Set(layout.cells.map((c) => c.id));
    let changed = false;
    for (const item of images) {
      const u = !used.has(item.id);
      if (u !== item.unused) { item.unused = u; changed = true; }
    }
    if (changed) renderGallery();
    els.download.disabled = false;
    const unusedCount = images.length - used.size;
    els.status.textContent = `${layout.cells.length} used · ${unusedCount} unused · ${Math.round(performance.now() - t0)} ms`;
  });
}

function clearCanvas() {
  const ctx = els.canvas.getContext('2d');
  ctx.clearRect(0, 0, els.canvas.width, els.canvas.height);
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
