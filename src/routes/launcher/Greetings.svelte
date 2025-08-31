<script lang="ts">
    import { onMount } from 'svelte';
    import gsap from 'gsap';
    import SplitText from 'gsap/SplitText';

    const greetings = ['Hỏi gì đi!', 'Ask away!', '何でも聞いてね!!'];
    let idx = $state.raw(0);

    gsap.registerPlugin(SplitText);

    onMount(() => {
        const interval = setInterval(() => {
            idx = (idx + 1) % greetings.length;
        }, 4000);
        return () => {
            clearInterval(interval);
        };
    });
</script>

{#key idx}
    <p
        {@attach (node) => {
            const split = SplitText.create(node, { type: 'chars' });
            gsap.from(split.chars, {
                duration: 0.4,
                xPercent: 25,
                autoAlpha: 0,
                stagger: 0.02,
            });
        }}
        class="text-base-fg-muted"
    >
        {greetings[idx]}
    </p>
{/key}
