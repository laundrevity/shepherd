<script lang="ts">
	import Canvas from '$lib/Canvas.svelte';
	import Explosion from '$lib/Explosion.svelte';

	import { invoke } from '@tauri-apps/api/tauri';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	let width: number;
	let height: number;

	type Point = [number, number];
	let explosions = [];

	let canvasElement;

	function handleCanvasMounted(canvas) {
		canvasElement = canvas;
	}

	// Reactive statement to update explosions when canvasElement changes
	$: if (canvasElement) {
		// Update the positions of existing explosions, if necessary
		explosions = explosions.map((e) => createExplosionProps(e));
	}

	onMount(() => {
		invoke('event_loop');

		listen('explode', (event) => {
			// console.log('explosion payload: ', event.payload);

			// Access the Point property which is an array
			const [x, y] = event.payload.Point;
			console.log('adding explosion at (x,y): ', x, y);
			explosions = [...explosions, { x, y, id: Math.random() }];
		});

		(async () => {
			const size = (await invoke('get_window_size')) as number[];
			width = size[0];
			height = size[1];
		})();

		const handleKeyDown = (event: KeyboardEvent) => {
			if (['w', 'a', 's', 'd'].includes(event.key)) {
				invoke('key_down', { key: event.key });
			}
			if (event.key === ' ') {
				invoke('toggle_pause');
				event.preventDefault(); // Prevent default action of the spacebar
			}
		};

		const handleKeyUp = (event: KeyboardEvent) => {
			if (['w', 'a', 's', 'd'].includes(event.key)) {
				invoke('key_up', { key: event.key });
			}
		};

		window.addEventListener('keydown', handleKeyDown);
		window.addEventListener('keyup', handleKeyUp);

		return () => {
			window.removeEventListener('keydown', handleKeyDown);
			window.removeEventListener('keyup', handleKeyUp);
		};
	});

	function removeExplosion(id) {
		explosions = explosions.filter((e) => e.id !== id);
	}

	function createExplosionProps(explosion) {
		return {
			x: explosion.x,
			y: explosion.y,
			onAnimationEnd: () => removeExplosion(explosion.id)
		};
	}
</script>

<main style="position: relative;">
	<!-- <Explosion x={100} y={100} onAnimationEnd={() => {}} /> -->
	<Canvas {width} {height} onCanvasMounted={handleCanvasMounted} />
	{#each explosions as explosion (explosion.id)}
		<Explosion {...createExplosionProps(explosion)} />
	{/each}
</main>
