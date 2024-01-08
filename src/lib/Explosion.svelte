<script lang="ts">
	export let x: number;
	export let y: number;
	export let radius: number;
	export let onAnimationEnd: Function;

	// Trigger the cleanup after the animation ends
	function handleAnimationEnd() {
		onAnimationEnd();
	}
</script>

<div
	class="explosion"
	on:animationend={handleAnimationEnd}
	style="left: {x - radius}px; top: {y - radius}px; width: {2 * radius}px; height: {2 * radius}px;"
></div>

<style>
	.explosion {
		position: absolute;
		background: radial-gradient(
			circle,
			rgba(255, 255, 0, 1) 0%,
			rgba(255, 165, 0, 1) 70%,
			rgba(255, 0, 0, 0) 100%
		);
		border-radius: 50%;
		animation: explode 0.5s ease-out;
		z-index: 1000;
	}

	@keyframes explode {
		from {
			transform: scale(0);
			opacity: 1;
		}
		to {
			transform: scale(1);
			opacity: 0;
		}
	}
</style>
