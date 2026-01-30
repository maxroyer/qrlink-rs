const app = (() => {
    const API_BASE = '/api/v1';
    
    let mode = 'link';
    let ttl = '1_week';

    const init = () => {
        initTheme();
        initModeToggle();
        initForm();
        
        // Initialize Lucide icons
        if (typeof lucide !== 'undefined') {
            lucide.createIcons();
        }
    };

    const initTheme = () => {
        const themeBtn = document.getElementById('theme-toggle');
        const html = document.documentElement;
        
        // Initial theme: use saved or detect from system
        let currentTheme = localStorage.getItem('theme');
        if (!currentTheme) {
            currentTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
        }
        
        html.setAttribute('data-theme', currentTheme);
        
        // Theme toggle: only toggle between dark and light
        themeBtn.addEventListener('click', () => {
            currentTheme = currentTheme === 'dark' ? 'light' : 'dark';
            html.setAttribute('data-theme', currentTheme);
            localStorage.setItem('theme', currentTheme);
        });
    };

    const initModeToggle = () => {
        const modeBtns = document.querySelectorAll('.mode-btn');
        const ttlOptions = document.getElementById('ttl-options');
        const submitBtn = document.getElementById('submit-btn');
        
        modeBtns.forEach(btn => {
            btn.addEventListener('click', () => {
                // Update active state
                modeBtns.forEach(b => b.classList.remove('active'));
                btn.classList.add('active');
                
                // Update mode
                mode = btn.dataset.mode;
                
                // Toggle TTL and button text
                if (mode === 'link') {
                    ttlOptions.style.display = 'block';
                    submitBtn.textContent = 'Generate Short Link';
                } else {
                    ttlOptions.style.display = 'none';
                    submitBtn.textContent = 'Generate QR Code';
                }
            });
        });
    };

    const initForm = () => {
        const form = document.getElementById('form');
        const urlInput = document.getElementById('url');
        const result = document.getElementById('result');
        const error = document.getElementById('error');
        const loading = document.getElementById('loading');

        // TTL change handler
        document.querySelectorAll('input[name="ttl"]').forEach(radio => {
            radio.addEventListener('change', (e) => {
                ttl = e.target.value;
            });
        });

        // Form submit handler
        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            error.style.display = 'none';
            
            const url = urlInput.value.trim();
            if (!url) {
                error.textContent = 'Please enter a URL';
                error.style.display = 'block';
                return;
            }

            loading.style.display = 'block';
            result.style.display = 'none';

            try {
                if (mode === 'qr') {
                    await generateQR(url, result, loading);
                } else {
                    await generateLink(url, result, loading);
                }
            } catch (err) {
                error.textContent = err.message || 'An error occurred';
                error.style.display = 'block';
                loading.style.display = 'none';
            }
        });
    };

    const generateQR = async (url, result, loading) => {
        const response = await fetch(`${API_BASE}/qr`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ url })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Failed to generate QR code');
        }

        const blob = await response.blob();
        const qrUrl = URL.createObjectURL(blob);
        
        // Store blob for sharing
        window.currentQRBlob = blob;

        result.innerHTML = `
            <h3>Your QR Code</h3>
            <img src="${qrUrl}" alt="QR Code">
            <div class="actions">
                <a href="${qrUrl}" download="qr-code.png">Download QR</a>
                <button onclick="app.shareQR()">Share</button>
                <button onclick="app.createAnother()">Create Another</button>
            </div>
        `;
        
        loading.style.display = 'none';
        result.style.display = 'block';
    };

    const generateLink = async (url, result, loading) => {
        const payload = { url };
        if (ttl !== 'never') {
            payload.ttl = ttl;
        }

        const response = await fetch(`${API_BASE}/links`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Failed to create short link');
        }

        const data = await response.json();

        let expiryHtml = '';
        if (data.expires_at) {
            const expiryDate = new Date(data.expires_at).toLocaleDateString();
            expiryHtml = `<p><strong>Expires:</strong> ${expiryDate}</p>`;
        }

        result.innerHTML = `
            <h3>Your Short Link</h3>
            <div class="url-display">
                <input type="text" value="${data.short_url}" readonly class="input">
                <button onclick="app.copy('${data.short_url}', this)">Copy</button>
            </div>
            <div class="info">
                <p><strong>Original:</strong> ${data.target_url}</p>
                ${expiryHtml}
            </div>
            <div class="actions">
                <a href="${data.short_url}" target="_blank">Visit Link</a>
                <button onclick="app.createAnother()">Create Another</button>
            </div>
        `;

        loading.style.display = 'none';
        result.style.display = 'block';
    };

    const copy = async (text, btn) => {
        try {
            await navigator.clipboard.writeText(text);
            const original = btn.textContent;
            btn.textContent = 'Copied!';
            setTimeout(() => btn.textContent = original, 2000);
        } catch (err) {
            alert('Failed to copy to clipboard');
        }
    };

    const createAnother = () => {
        document.getElementById('url').value = '';
        document.getElementById('result').style.display = 'none';
        document.getElementById('error').style.display = 'none';
    };

    const shareQR = async () => {
        const blob = window.currentQRBlob;
        if (!blob) return;
        
        if (navigator.share && navigator.canShare) {
            const file = new File([blob], 'qr-code.png', { type: 'image/png' });
            const shareData = { files: [file] };
            
            if (navigator.canShare(shareData)) {
                try {
                    await navigator.share(shareData);
                } catch (err) {
                    if (err.name !== 'AbortError') {
                        console.error('Share failed:', err);
                    }
                }
                return;
            }
        }
        
        // Fallback: download the image
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'qr-code.png';
        a.click();
        URL.revokeObjectURL(url);
    };

    return { init, copy, createAnother, shareQR };
})();

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', app.init);
} else {
    app.init();
}
