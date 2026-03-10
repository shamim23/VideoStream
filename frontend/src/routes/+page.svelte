<script>
	let file = null;
	let uploading = false;
	let uploadProgress = 0;
	let shareUrl = null;
	let videoId = null;
	let error = null;
	let uploadSpeed = '';
	let uploadEta = '';
	let startTime = 0;
	const apiBaseUrl = import.meta.env.PUBLIC_API_BASE_URL ?? '';

	function buildApiUrl(path) {
		const base = apiBaseUrl.replace(/\/+$/, '');
		const normalizedPath = path.startsWith('/') ? path : `/${path}`;
		if (!base) return normalizedPath;
		if (base.endsWith('/api') && normalizedPath.startsWith('/api/')) {
			return `${base}${normalizedPath.slice(4)}`;
		}
		return `${base}${normalizedPath}`;
	}

	function handleFileSelect(event) {
		file = event.target.files[0];
		error = null;
		shareUrl = null;
		videoId = null;
	}

	function uploadVideo() {
		if (!file) {
			error = 'Please select a video file';
			return;
		}

		uploading = true;
		uploadProgress = 0;
		uploadSpeed = '';
		uploadEta = '';
		startTime = Date.now();
		error = null;

		const formData = new FormData();
		formData.append('video', file);

		const xhr = new XMLHttpRequest();

		xhr.upload.addEventListener('progress', (event) => {
			if (event.lengthComputable) {
				uploadProgress = Math.round((event.loaded / event.total) * 100);
				
				// Calculate speed
				const elapsed = (Date.now() - startTime) / 1000;
				const speed = event.loaded / elapsed;
				uploadSpeed = formatSpeed(speed);
				
				// Calculate ETA
				const remaining = event.total - event.loaded;
				const etaSeconds = remaining / speed;
				uploadEta = formatEta(etaSeconds);
			}
		});

		xhr.addEventListener('load', () => {
			if (xhr.status >= 200 && xhr.status < 300) {
				try {
					const data = JSON.parse(xhr.responseText);
					shareUrl = data.share_url;
					videoId = data.share_url?.split('/').pop() ?? null;
					uploadProgress = 100;
				} catch (e) {
					error = 'Invalid server response';
				}
			} else {
				error = `Upload failed: ${xhr.status} ${xhr.statusText}`;
			}
			uploading = false;
		});

		xhr.addEventListener('error', () => {
			error = 'Upload failed. Please try again.';
			uploading = false;
		});

		xhr.addEventListener('abort', () => {
			error = 'Upload cancelled.';
			uploading = false;
		});

		xhr.open('POST', buildApiUrl('/api/upload'));
		xhr.send(formData);
	}

	function cancelUpload() {
		// Note: xhr is local to uploadVideo, so we'd need to store it to cancel
		// For now, just reset the UI
		uploading = false;
		uploadProgress = 0;
	}

	function getFullWatchUrl() {
		if (!videoId) return '';
		return `${window.location.origin}/watch/${videoId}`;
	}

	function getVideoStreamUrl() {
		if (!shareUrl) return '';
		return buildApiUrl(shareUrl);
	}

	function copyLink() {
		navigator.clipboard.writeText(getFullWatchUrl());
		alert('Link copied to clipboard!');
	}

	function formatSpeed(bytesPerSecond) {
		if (bytesPerSecond === 0) return '';
		if (bytesPerSecond < 1024) return `${bytesPerSecond.toFixed(1)} B/s`;
		if (bytesPerSecond < 1024 * 1024) return `${(bytesPerSecond / 1024).toFixed(1)} KB/s`;
		return `${(bytesPerSecond / (1024 * 1024)).toFixed(1)} MB/s`;
	}

	function formatEta(seconds) {
		if (!isFinite(seconds) || seconds < 0) return '';
		if (seconds < 60) return `${Math.round(seconds)}s remaining`;
		const minutes = Math.floor(seconds / 60);
		const secs = Math.round(seconds % 60);
		return `${minutes}m ${secs}s remaining`;
	}
</script>

<main>
	<h1>🎬 Video Streaming Service</h1>
	
	<div class="upload-section">
		<h2>Upload Video</h2>
		
		<div class="file-input">
			<input 
				type="file" 
				accept="video/*" 
				on:change={handleFileSelect}
				disabled={uploading}
			/>
			{#if file}
				<p class="file-name">Selected: {file.name} ({(file.size / 1024 / 1024).toFixed(2)} MB)</p>
			{/if}
		</div>

		<button 
			on:click={uploadVideo} 
			disabled={!file || uploading}
			class="upload-btn"
		>
			{#if uploading}
				Uploading... {uploadProgress}%
			{:else}
				Upload Video
			{/if}
		</button>

		{#if uploading}
			<div class="progress-container">
				<div class="progress-bar">
					<div class="progress" style="width: {uploadProgress}%"></div>
				</div>
				<div class="progress-info">
					<span class="progress-percent">{uploadProgress}%</span>
					{#if uploadSpeed}
						<span class="progress-speed">{uploadSpeed}</span>
					{/if}
					{#if uploadEta}
						<span class="progress-eta">{uploadEta}</span>
					{/if}
				</div>
			</div>
		{/if}

		{#if error}
			<p class="error">{error}</p>
		{/if}
	</div>

	{#if shareUrl}
		<div class="success-section">
			<h2>✅ Upload Complete!</h2>
			
			<div class="share-link">
				<p>Share URL:</p>
				<code>{getFullWatchUrl()}</code>
				<button on:click={copyLink} class="copy-btn">Copy Link</button>
			</div>

			<div class="player-section">
				<h3>Preview:</h3>
				<!-- svelte-ignore a11y-media-has-caption -->
				<video 
					src={getVideoStreamUrl()}
					controls 
					width="100%"
				>
					Your browser does not support the video tag.
				</video>
			</div>
		</div>
	{/if}
</main>

<style>
	:global(body) {
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
		margin: 0;
		padding: 0;
		background: #f5f5f5;
	}

	main {
		max-width: 800px;
		margin: 0 auto;
		padding: 2rem;
	}

	h1 {
		text-align: center;
		color: #333;
		margin-bottom: 2rem;
	}

	.upload-section, .success-section {
		background: white;
		padding: 2rem;
		border-radius: 8px;
		box-shadow: 0 2px 4px rgba(0,0,0,0.1);
		margin-bottom: 2rem;
	}

	.file-input {
		margin-bottom: 1rem;
	}

	.file-input input {
		padding: 0.5rem;
	}

	.file-name {
		color: #666;
		font-size: 0.9rem;
		margin-top: 0.5rem;
	}

	.upload-btn, .copy-btn {
		background: #4CAF50;
		color: white;
		border: none;
		padding: 0.75rem 1.5rem;
		border-radius: 4px;
		cursor: pointer;
		font-size: 1rem;
		transition: background 0.3s;
	}

	.upload-btn:hover:not(:disabled) {
		background: #45a049;
	}

	.upload-btn:disabled {
		background: #ccc;
		cursor: not-allowed;
	}

	.progress-container {
		margin-top: 1rem;
	}

	.progress-bar {
		width: 100%;
		height: 20px;
		background: #e0e0e0;
		border-radius: 10px;
		overflow: hidden;
	}

	.progress {
		height: 100%;
		background: #4CAF50;
		transition: width 0.2s;
	}

	.progress-info {
		display: flex;
		justify-content: space-between;
		margin-top: 0.5rem;
		font-size: 0.85rem;
		color: #666;
	}

	.progress-percent {
		font-weight: bold;
		color: #333;
	}

	.progress-speed {
		color: #2196F3;
	}

	.progress-eta {
		color: #666;
	}

	.error {
		color: #f44336;
		margin-top: 1rem;
	}

	.share-link {
		background: #f5f5f5;
		padding: 1rem;
		border-radius: 4px;
		margin: 1rem 0;
	}

	.share-link code {
		word-break: break-all;
		display: block;
		margin: 0.5rem 0;
		padding: 0.5rem;
		background: #e0e0e0;
		border-radius: 4px;
	}

	.copy-btn {
		background: #2196F3;
	}

	.copy-btn:hover {
		background: #1976D2;
	}

	.player-section {
		margin-top: 2rem;
	}

	.player-section video {
		border-radius: 4px;
		max-width: 100%;
	}
</style>
