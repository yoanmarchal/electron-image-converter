import params from '@params';

const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];

/* ---------- Thème clair / sombre ---------- */
$('[data-theme-toggle]')?.addEventListener('click', () => {
  const dark = document.documentElement.classList.toggle('dark');
  try { localStorage.setItem('theme', dark ? 'dark' : 'light'); } catch { /* stockage indisponible */ }
});

/* ---------- En-tête : fond au défilement ---------- */
const header = $('[data-header]');
const onScroll = () => header?.toggleAttribute('data-scrolled', window.scrollY > 8);
onScroll();
window.addEventListener('scroll', onScroll, { passive: true });

/* ---------- Apparition au défilement ---------- */
const revealed = $$('[data-reveal]');
if ('IntersectionObserver' in window) {
  const io = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      entry.target.classList.add('is-visible');
      io.unobserve(entry.target);
    }
  }, { rootMargin: '0px 0px -8% 0px' });
  revealed.forEach((el) => io.observe(el));
} else {
  revealed.forEach((el) => el.classList.add('is-visible'));
}

/* ---------- Démo du réglage de qualité (mêmes libellés que l'application) ---------- */
const demo = $('[data-quality-demo]');
if (demo) {
  const range = $('input[type=range]', demo);
  const value = $('[data-quality-value]', demo);
  const hint = $('[data-quality-hint]', demo);
  const buttons = $$('[data-format]', demo);
  let format = 'webp';

  const render = () => {
    const q = Number(range.value);
    const lossless = format === 'png';
    range.disabled = lossless;
    range.style.setProperty('--fill', `${q}%`);
    value.textContent = lossless ? '100 %' : `${q} %`;
    hint.textContent = lossless
      ? "Le PNG est sans perte : la qualité ne s'applique pas"
      : q < 30 ? 'Qualité basse, fichier plus petit'
      : q < 70 ? 'Qualité équilibrée'
      : 'Haute qualité, fichier plus grand';
  };

  range.addEventListener('input', render);
  buttons.forEach((btn) => btn.addEventListener('click', () => {
    format = btn.dataset.format;
    buttons.forEach((b) => b.setAttribute('aria-checked', String(b === btn)));
    render();
  }));
  render();
}

/* ---------- Détection du système ---------- */
function detectOS() {
  const platform = (navigator.userAgentData?.platform || navigator.platform || '').toLowerCase();
  const ua = navigator.userAgent.toLowerCase();
  if (/android|iphone|ipad|ipod/.test(ua)) return null;
  if (platform.includes('win') || ua.includes('windows')) return 'windows';
  if (platform.includes('linux') || ua.includes('linux')) return 'linux';
  return null;
}

const os = detectOS();
const PRIMARY = { windows: 'win-x64', linux: 'deb-x64' };
const OS_LABEL = { windows: 'Windows', linux: 'Linux' };

if (os) {
  const card = $(`[data-os-card="${os}"]`);
  card?.setAttribute('data-current', '');
  $('[data-current-badge]', card)?.classList.remove('hidden');
  const label = $('[data-primary-label]');
  if (label) label.textContent = `Télécharger pour ${OS_LABEL[os]}`;
}

/* ---------- Liens directs vers la dernière release GitHub ---------- */
const formatSize = (bytes) => `${(bytes / 1024 / 1024).toLocaleString('fr-FR', { maximumFractionDigits: 0 })} Mo`;

async function loadRelease() {
  const res = await fetch(`https://api.github.com/repos/${params.repo}/releases/latest`, {
    headers: { Accept: 'application/vnd.github+json' },
  });
  if (!res.ok) throw new Error(`GitHub API ${res.status}`);
  return res.json();
}

loadRelease().then((release) => {
  const version = release.tag_name.replace(/^v/, '');
  $$('[data-version]').forEach((el) => { el.textContent = version; });

  const date = release.published_at && new Date(release.published_at);
  const dateEl = $('[data-release-date]');
  if (date && dateEl) dateEl.textContent = `, publiée le ${date.toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' })}`;

  const assetUrls = {};
  $$('[data-asset]').forEach((link) => {
    const re = new RegExp(link.dataset.match, 'i');
    const asset = release.assets.find((a) => re.test(a.name));
    if (!asset) return;
    link.href = asset.browser_download_url;
    link.title = asset.name;
    const size = $('[data-asset-size]', link);
    if (size) size.textContent = formatSize(asset.size);
    assetUrls[link.dataset.asset] = asset.browser_download_url;
  });

  // Le bouton principal télécharge directement le bon fichier quand le système est reconnu.
  const primary = $('[data-primary-download]');
  const url = os && assetUrls[PRIMARY[os]];
  if (primary && url) {
    primary.href = url;
    const meta = $('[data-primary-meta]');
    if (meta) {
      meta.innerHTML = '';
      meta.append(`v${version} · ${os === 'windows' ? 'Installateur .exe · x64' : 'Paquet .deb · x64'} · `);
      const other = Object.assign(document.createElement('a'), {
        href: '#telecharger',
        textContent: 'Autres plateformes',
        className: 'underline decoration-zinc-400/50 underline-offset-4 hover:text-zinc-900 dark:hover:text-white',
      });
      meta.append(other);
    }
  }
}).catch((err) => {
  // Les liens restent pointés vers la page des releases.
  console.warn('Release GitHub indisponible :', err);
});
