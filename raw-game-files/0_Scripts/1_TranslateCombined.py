import json
import os
import asyncio
import aiohttp

INPUT_FILE = "../3_Combined/Combined.json"
OUTPUT_FILE = "../3_Combined/CombinedTranslated.json"
BATCH_SIZE = 500
CONCURRENT_BATCHES = 5

TRANSLATE_URL = "https://translate.googleapis.com/translate_a/single?client=gtx&sl=zh-CN&tl=en&dt=t&q="

# Load JSON
with open(INPUT_FILE, "r", encoding="utf-8") as f:
    data = json.load(f)

if os.path.exists(OUTPUT_FILE):
    with open(OUTPUT_FILE, "r", encoding="utf-8") as f:
        translated_data = json.load(f)
else:
    translated_data = {}

# Flatten all ChineseShorts
to_translate = []
for key, sources in data.items():
    for source_name, fields in sources.items():
        if "ChineseShort" in fields:
            if key in translated_data and source_name in translated_data[key] and "AIEnglishShort" in translated_data[key][source_name]:
                continue
            to_translate.append((key, source_name, fields["ChineseShort"]))

total = len(to_translate)
counter = 0
counter_lock = asyncio.Lock()

async def translate_text(session, item):
    global counter
    key, source_name, original_text = item
    url = TRANSLATE_URL + aiohttp.helpers.quote(original_text)
    async with session.get(url) as resp:
        result = await resp.json()
        translated_text = result[0][0][0]

        # Save immediately
        if key not in translated_data:
            translated_data[key] = {}
        if source_name not in translated_data[key]:
            translated_data[key][source_name] = {}
        translated_data[key][source_name]["AIEnglishShort"] = translated_text

        # Update counter safely
        async with counter_lock:
            counter += 1
            print(f"{counter} / {total}: {original_text} -> {translated_text}")

        return translated_text

async def translate_batch(session, batch):
    # Translate all items concurrently within the batch
    tasks = [asyncio.create_task(translate_text(session, item)) for item in batch]
    await asyncio.gather(*tasks)

    # Save batch after all items finished
    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        json.dump(translated_data, f, ensure_ascii=False, indent=4)
    print(f"Saved batch of {len(batch)} items.\n")

async def translated_combined():
    connector = aiohttp.TCPConnector(limit_per_host=CONCURRENT_BATCHES)
    async with aiohttp.ClientSession(connector=connector) as session:
        for i in range(0, len(to_translate), BATCH_SIZE):
            batch = to_translate[i:i+BATCH_SIZE]
            await translate_batch(session, batch)
    print("All translations complete!")

def add_manual_override():
    SRC_FILE = "../3_Combined/CombinedTranslated.json"
    DEST_FILE = "../4_final/CombinedTranslatedWithManualOverrides.json"

    # Ensure destination folder exists
    os.makedirs(os.path.dirname(DEST_FILE), exist_ok=True)

    # Load the translated JSON
    with open(SRC_FILE, "r", encoding="utf-8") as f:
        data = json.load(f)

    # Add top-level keys if missing
    for key, entry in data.items():
        if "EnglishShortManualOverride" not in entry:
            entry["EnglishShortManualOverride"] = None
        if "Comment" not in entry:
            entry["Comment"] = None

    # Save to new location
    with open(DEST_FILE, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=4)

    print(f"Copied with manual override placeholders to {DEST_FILE}")

if __name__ == "__main__":
    asyncio.run(translated_combined())
    add_manual_override()

