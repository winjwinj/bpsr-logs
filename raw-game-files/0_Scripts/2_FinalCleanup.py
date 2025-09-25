import json

# Input and output paths
input_file = "../4_Final/CombinedtranslatedWithManualOverrides.json"
output_file = "../../src-tauri/meter-data/SkillName.json"
conflict_file = "../4_Final/Conflicts.json"

# Priority order for flattening
priority = [
    ("EnglishShortManualOverride", None),

    ("RecountTable_Clean.json", "EnglishShort"),
    ("SkillTable_Clean.json", "EnglishShort"),

    ("skill_names_Clean.json", "AIEnglishShort"),
    ("RecountTable_Clean.json", "AIEnglishShort"),
    ("SkillTable_Clean.json", "AIEnglishShort"),

    ("skill_names_Clean.json", "ChineseShort"),
    ("RecountTable_Clean.json", "ChineseShort"),
    ("SkillTable_Clean.json", "ChineseShort"),

    ("BuffTable_Clean.json", "EnglishShort"),
    ("BuffTable_Clean.json", "AIEnglishShort")
]

# Fields to check for conflicts
english_fields_to_check = [
    ("RecountTable_Clean.json", "EnglishShort"),
    ("SkillTable_Clean.json", "EnglishShort"),
    ("skill_names_Clean.json", "EnglishShort"),
    ("BuffTable_Clean.json", "EnglishShort")
]

# Load the input JSON
with open(input_file, "r", encoding="utf-8") as f:
    data = json.load(f)

cleaned = {}
conflicts = {}

for key, value in data.items():
    # 1. Check for conflicts in EnglishShort fields
    english_values = set()
    for field, subfield in english_fields_to_check:
        val = value.get(field, {}).get(subfield)
        if val:
            english_values.add(val)

    if len(english_values) > 1:
        conflicts[key] = value

    # 2. Flatten according to priority
    selected = None
    for field, subfield in priority:
        try:
            if subfield is None:
                val = value.get(field)
            else:
                val = value.get(field, {}).get(subfield)

            if val:
                # If this field is an AIEnglishShort, add suffix
                if subfield == "AIEnglishShort":
                    val = f"AI: {val}"
                selected = val
                break
        except AttributeError:
            continue
    if selected:
        cleaned[key] = selected

# Write cleaned JSON
with open(output_file, "w", encoding="utf-8") as f:
    json.dump(cleaned, f, ensure_ascii=False, indent=2)

# Write conflicts JSON
with open(conflict_file, "w", encoding="utf-8") as f:
    json.dump(conflicts, f, ensure_ascii=False, indent=2)

print(f"Saved cleaned JSON to {output_file}")
print(f"Saved conflicts JSON to {conflict_file} (total {len(conflicts)} conflicts)")
