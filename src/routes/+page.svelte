<script lang="ts">
	import Canvas from '$lib/Canvas.svelte';
	import Explosion from '$lib/Explosion.svelte';

	import { invoke } from '@tauri-apps/api/tauri';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	interface Explosion {
		x: number;
		y: number;
		id: number;
	}

	interface GameConstants {
		window_width: number;
		window_height: number;
		circle_radius: number;
		diamond_radius: number;
		triangle_radius: number;
		square_radius: number;
		explosion_radius: number;
	}

	type Point = [number, number];
	type explosionPayload = { Point?: Point };
	type ScoreMultiplierPayload = [number, number];

	let score = 0; // Example score
	let multiplier = 1; // Example multiplier

	let explosions: Explosion[] = [];
	let gameConstants: GameConstants;
	let constantsLoaded = false;

	let canvasElement: HTMLCanvasElement | null = null;

	function handleCanvasMounted(canvas: HTMLCanvasElement) {
		canvasElement = canvas;
	}

	onMount(() => {
		(async () => {
			gameConstants = (await invoke('get_game_constants')) as GameConstants;
			console.log('got GameConstants');
			constantsLoaded = true;
		})();

		invoke('event_loop');

		listen('explode', (event) => {
			// Access the Point property which is an array
			let explosionPayload = event.payload as explosionPayload;
			const [x, y] = explosionPayload.Point ?? [0, 0];
			console.log('adding explosion at (x,y): ', x, y);
			explosions = [...explosions, { x, y, id: Math.random() }];
		});

		listen('update_score_multiplier', (event) => {
			const [updatedScore, updatedMultiplier] = event.payload as ScoreMultiplierPayload;
			score = updatedScore;
			multiplier = updatedMultiplier;
		});

		const handleKeyDown = (event: KeyboardEvent) => {
			if (['w', 'a', 's', 'd'].includes(event.key)) {
				invoke('key_down', { key: event.key });
			}
			if (event.key === ' ') {
				console.log('handling spacebar');
				invoke('handle_spacebar');
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

	function removeExplosion(id: number) {
		explosions = explosions.filter((e) => e.id !== id);
	}

	function createExplosionProps(explosion: Explosion) {
		return {
			x: explosion.x,
			y: explosion.y,
			radius: gameConstants.explosion_radius,
			onAnimationEnd: () => removeExplosion(explosion.id)
		};
	}
</script>

<main style="position: relative;">
	{#if constantsLoaded}
		<Canvas {...gameConstants} {score} {multiplier} onCanvasMounted={handleCanvasMounted} />
		{#each explosions as explosion (explosion.id)}
			<Explosion {...createExplosionProps(explosion)} />
		{/each}
	{/if}
</main>
