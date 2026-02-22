// Script pour récupérer dynamiquement la dernière version depuis GitHub
// À ajouter optionnellement dans le baseof.html

(function() {
    'use strict';

    const GITHUB_REPO = 'yoanmarchal/electron-image-converter';
    
    // Récupérer les informations de la dernière release
    async function getLatestRelease() {
        try {
            const response = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`);
            if (!response.ok) {
                throw new Error('Failed to fetch release info');
            }
            return await response.json();
        } catch (error) {
            console.error('Error fetching latest release:', error);
            return null;
        }
    }

    // Mettre à jour les liens de téléchargement
    async function updateDownloadLinks() {
        const release = await getLatestRelease();
        
        if (!release || !release.assets) {
            console.warn('No release data available, using default links');
            return;
        }

        const version = release.tag_name.replace('v', '');
        
        // Trouver les assets Windows et Linux
        const windowsAsset = release.assets.find(asset => 
            asset.name.toLowerCase().endsWith('.exe') && 
            asset.name.toLowerCase().includes('setup')
        );
        
        const linuxAsset = release.assets.find(asset => 
            asset.name.endsWith('.deb') && 
            asset.name.includes('amd64')
        );

        // Mettre à jour les liens Windows
        if (windowsAsset) {
            const windowsLinks = document.querySelectorAll('a[data-download="windows"]');
            windowsLinks.forEach(link => {
                link.href = windowsAsset.browser_download_url;
                link.setAttribute('data-version', version);
                // Ajouter un badge de version si souhaité
                link.setAttribute('title', `Télécharger v${version} pour Windows`);
            });
            console.log(`✅ Windows download link updated to version ${version}: ${windowsAsset.name}`);
        }

        // Mettre à jour les liens Linux
        if (linuxAsset) {
            const linuxLinks = document.querySelectorAll('a[data-download="linux"]');
            linuxLinks.forEach(link => {
                link.href = linuxAsset.browser_download_url;
                link.setAttribute('data-version', version);
                link.setAttribute('title', `Télécharger v${version} pour Linux`);
            });
            console.log(`✅ Linux download link updated to version ${version}: ${linuxAsset.name}`);
        }

        // Mettre à jour les badges de version si présents
        const versionBadges = document.querySelectorAll('[data-version-badge]');
        versionBadges.forEach(badge => {
            badge.textContent = `v${version}`;
        });
    }

    // Exécuter au chargement de la page
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', updateDownloadLinks);
    } else {
        updateDownloadLinks();
    }
})();
