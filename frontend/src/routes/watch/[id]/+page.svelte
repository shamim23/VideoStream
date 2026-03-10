<script>
	import { page } from '$app/stores';
	
	const videoId = $page.params.id;
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

	const videoUrl = buildApiUrl(`/api/watch/${videoId}`);
</script>

<main>
	<div class="video-container">
		<!-- svelte-ignore a11y-media-has-caption -->
		<video 
			src={videoUrl} 
			controls 
			width="100%"
			autoplay
		>
			Your browser does not support the video tag.
		</video>
	</div>
	
	<div class="info">
		<h1>Video Player</h1>
		<p>Video ID: <code>{videoId}</code></p>
		<a href="/">← Upload another video</a>
	</div>
</main>

<style>
	:global(body) {
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
		margin: 0;
		padding: 0;
		background: #000;
	}

	main {
		max-width: 1200px;
		margin: 0 auto;
		padding: 1rem;
	}

	.video-container {
		background: #000;
		border-radius: 8px;
		overflow: hidden;
	}

	video {
		display: block;
		max-height: 80vh;
		margin: 0 auto;
	}

	.info {
		background: #1a1a1a;
		color: #fff;
		padding: 1.5rem;
		border-radius: 8px;
		margin-top: 1rem;
	}

	.info h1 {
		margin: 0 0 0.5rem 0;
	}

	.info p {
		color: #999;
		margin: 0.5rem 0;
	}

	.info code {
		background: #333;
		padding: 0.2rem 0.4rem;
		border-radius: 4px;
	}

	.info a {
		color: #4CAF50;
		text-decoration: none;
		display: inline-block;
		margin-top: 1rem;
	}

	.info a:hover {
		text-decoration: underline;
	}
</style>
