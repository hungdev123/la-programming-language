document.getElementById("year").textContent = new Date().getFullYear();

// ---- Reveal animation on load ----
window.addEventListener('DOMContentLoaded', () => {
  const reveals = document.querySelectorAll('.reveal');
  reveals.forEach((el, i) => {
    setTimeout(() => el.classList.add('visible'), 80 * i);
  });

  // set latest official/community download links (no version selector)
  const latest = {
    ver: '1.0.0',
    official: {
      linux: 'https://github.com/hungdev123/la-programming-language/releases/download/Install/la-1.0.0.zip',
      windows: 'downloads/la-1.0.0-official-windows.zip'
    },
    community: {
      linux: 'downloads/la-1.0.0-community-linux.tar.gz',
      windows: 'downloads/la-1.0.0-community-windows.zip'
    }
  };

  const officialBtn = document.getElementById('downloadOfficialBtn');
  const communityBtn = document.getElementById('downloadCommunityBtn');
  const currentVersionEl = document.getElementById('currentVersion');
  const osButtons = document.querySelectorAll('.os-btn');

  function setOs(os) {
    if (!officialBtn || !communityBtn) return;
    officialBtn.href = latest.official[os];
    communityBtn.href = latest.community[os];
    // set download attribute proper extension if needed
    officialBtn.setAttribute('download', `la-${latest.ver}-${os}`);
    communityBtn.setAttribute('download', `la-${latest.ver}-${os}-community`);
    // active class on buttons
    osButtons.forEach(b => b.classList.toggle('active', b.dataset.os === os));
  }

  if (officialBtn && communityBtn && currentVersionEl) {
    currentVersionEl.textContent = latest.ver;
    // detect platform default
    const isWin = navigator.platform.toLowerCase().includes('win');
    const defaultOs = isWin ? 'windows' : 'linux';
    setOs(defaultOs);
  }

  osButtons.forEach(b => {
    b.addEventListener('click', () => setOs(b.dataset.os));
  });
});
