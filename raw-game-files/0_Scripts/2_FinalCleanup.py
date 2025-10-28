import json

def cleanup_skill_names():
    # Input and output paths
    input_file = "../4_Final/CombinedtranslatedWithManualOverrides.json"
    output_file = "../../src/lib/data/json/SkillName.json"
    conflict_file = "../4_Final/Conflicts.json"

    # Priority order for flattening
    priority = [
        ("EnglishShortManualOverride", None),

        ("skills_questlog_clean.json", "EnglishShort"),

        ("SkillTable_Clean.json", "EnglishShort"),
        ("RecountTable_Clean.json", "EnglishShort"),

        ("skill_names_Clean.json", "AIEnglishShort"),
        ("SkillTable_Clean.json", "AIEnglishShort"),
        ("RecountTable_Clean.json", "AIEnglishShort"),

        ("skill_names_Clean.json", "ChineseShort"),
        ("SkillTable_Clean.json", "ChineseShort"),
        ("RecountTable_Clean.json", "ChineseShort"),

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

"""
Reads skills_questlog_clean.json, extracts skill_uid:icon mapping, and writes it to output_path as JSON.
"""
def generate_skill_icon_json(
        input_file="../2_Clean/skills_questlog_clean.json",
        output_file="../../src/lib/data/json/SkillIcon.json"
):

    with open(input_file, "r", encoding="utf-8") as f:
        skills = json.load(f)
    icon_map = {uid: entry.get("icon").split("/")[-1] for uid, entry in skills.items() if entry.get("icon")}
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(icon_map, f, ensure_ascii=False, indent=2)
    print(f"Saved skill_uid to icon mapping to {output_file}")

def cleanup_monster_names(
        input_file="../1_Dirty/monsters_questlog.json",
        output_file="../../src/lib/data/json/MonsterName.json",
        boss_output_file="../../src/lib/data/json/MonsterNameBoss.json",
        language="en"
):
    """
    Reads monsters_questlog.json, extracts id:name mapping, and writes it to output_file as JSON.
    Only includes entries where language is 'en' (English).
    If category is 'boss', also adds to boss_output_file.
    """
    with open(input_file, "r", encoding="utf-8") as f:
        data = json.load(f)
    # Try to handle both list and dict formats
    mapping = {}
    boss_mapping = {}
    if isinstance(data, dict):
        for k, v in data.items():
            if v.get("language") == language and "name" in v:
                mapping[str(k)] = v["name"]
                if v.get("main_category") == "boss":
                    boss_mapping[str(k)] = v["name"]
    elif isinstance(data, list):
        for entry in data:
            if entry.get("language") == language and "id" in entry and "name" in entry:
                mapping[str(entry["id"])] = entry["name"]
                if entry.get("main_category") == "boss":
                    boss_mapping[str(entry["id"])] = entry["name"]
    else:
        raise ValueError("Unexpected format in monsters_questlog.json")
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(mapping, f, ensure_ascii=False, indent=2)
    with open(boss_output_file, "w", encoding="utf-8") as f:
        json.dump(boss_mapping, f, ensure_ascii=False, indent=2)
    print(f"Saved monster id to name mapping to {output_file} (language={language})")
    print(f"Saved boss monster id to name mapping to {boss_output_file} (language={language})")

if __name__ == "__main__":
    cleanup_skill_names()
    generate_skill_icon_json()

    cleanup_monster_names()
