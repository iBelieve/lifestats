<script lang="ts" generics="T extends string">
	interface Option {
		value: T;
		label: string;
	}

	interface Props {
		options: Option[];
		value: T;
	}

	let { options, value = $bindable() }: Props = $props();
</script>

<div class="inline-flex rounded-lg bg-gray-100 p-1" role="group">
	{#each options as option (option.value)}
		<button
			type="button"
			class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors {value === option.value
				? 'bg-white text-gray-900 shadow-sm'
				: 'text-gray-600 hover:text-gray-900'}"
			aria-pressed={value === option.value}
			onclick={() => (value = option.value)}
		>
			{option.label}
		</button>
	{/each}
</div>
