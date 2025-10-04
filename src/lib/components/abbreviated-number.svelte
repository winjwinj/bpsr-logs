<script lang="ts">
  let { val = 0 }: { val: number } = $props();

  function abbreviateNumberSplit(n: number): [number, string] {
    if (n >= 1e3 && n < 1e6) return [+(n / 1e3).toFixed(1), "k"];
    if (n >= 1e6 && n < 1e9) return [+(n / 1e6).toFixed(1), "m"];
    if (n >= 1e9 && n < 1e12) return [+(n / 1e9).toFixed(1), "b"];
    if (n >= 1e12) return [+(n / 1e12).toFixed(1), "t"];
    else return [+n.toFixed(0), ""];
  }

  let val_arr = $derived(abbreviateNumberSplit(val));
</script>

{val_arr[0]}<span class="text-tiny text-gray-300">{val_arr[1]}</span>
