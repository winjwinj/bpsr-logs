export function formatElapsed(msBigInt: bigint) {
  const totalSeconds = Math.floor(Number(msBigInt) / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export function pct(part: bigint, whole: bigint) {
	if (!whole) return "0.00%";
	return `${((Number(part) / Number(whole)) * 100).toFixed(2)}%`;
}
