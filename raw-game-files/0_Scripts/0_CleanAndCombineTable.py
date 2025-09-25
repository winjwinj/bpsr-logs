import json
import \
    os
from collections import \
    OrderedDict


def clean_buff_table(input_path, input_file):
    full_input_path = os.path.join(input_path, input_file)
    with open(full_input_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    cleaned_data = {}
    for key, value in data.items():
        cleaned_entry = {}
        if value.get("NameDesign"):
            cleaned_entry["ChineseShort"] = value["NameDesign"]
        if value.get("Name$english"):
            cleaned_entry["EnglishShort"] = value["Name$english"]
        if value.get("Desc$english"):
            cleaned_entry["EnglishLong"] = value["Desc$english"]
        cleaned_data[key] = cleaned_entry

    base, ext = os.path.splitext(input_file)
    output_path = os.path.join(f"../2_Clean/{base}_Clean{ext}")
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(cleaned_data, f, ensure_ascii=False, indent=4)

    print(f"✅ Cleaned {full_input_path} saved to {output_path}")

def clean_recount_table(input_path, input_file):
    full_input_path = os.path.join(input_path, input_file)
    with open(full_input_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    cleaned_data = {}
    for key, value in data.items():
        cleaned_entry = {}
        if value.get("RecountName$english"):
            cleaned_entry["EnglishShort"] = value["RecountName$english"]
        cleaned_data[key] = cleaned_entry

    base, ext = os.path.splitext(input_file)
    output_path = os.path.join(f"../2_Clean/{base}_Clean{ext}")
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(cleaned_data, f, ensure_ascii=False, indent=4)

    print(f"✅ Cleaned {full_input_path} saved to {output_path}")

def clean_skill_names(input_path, input_file):
    full_input_path = os.path.join(input_path, input_file)
    with open(full_input_path, "r", encoding="utf-8") as f:
        data_chinese = json.load(f)

    # Merge data
    cleaned_data = {}
    for key in data_chinese.keys():  # sort numerically
        cleaned_data[key] = {}
        if key in data_chinese and data_chinese[key]:
            cleaned_data[key]["ChineseShort"] = data_chinese[key]

    base, ext = os.path.splitext(input_file)
    output_path = os.path.join(f"../2_Clean/{base}_Clean{ext}")
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(cleaned_data, f, ensure_ascii=False, indent=4)

    print(f"✅ Cleaned {full_input_path} saved to {output_path}")

def clean_english_skill_names(input_path, input_file):
    full_input_path = os.path.join(input_path, input_file)
    with open(input_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    cleaned_data = {key: {"EnglishShort": value} for key, value in data.items() if value}

    base, ext = os.path.splitext(input_file)
    output_path = os.path.join(f"../2_Clean/{base}_Clean{ext}")
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(cleaned_data, f, ensure_ascii=False, indent=4)

    print(f"✅ Cleaned {full_input_path} saved to {output_path}")

def clean_talent_table(input_path, input_file):
    full_input_path = os.path.join(input_path, input_file)
    with open(full_input_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    cleaned_data = {}
    for key, value in data.items():
        cleaned_entry = {}
        if value.get("Des"):
            cleaned_entry["ChineseShort"] = value["Des"]
        if value.get("TalentName$english"):
            cleaned_entry["EnglishShort"] = value["TalentName$english"]
        if value.get("TalentDes$english"):
            cleaned_entry["EnglishLong"] = value["TalentDes$english"]

        cleaned_data[key] = cleaned_entry

    base, ext = os.path.splitext(input_file)
    output_path = os.path.join(f"../2_Clean/{base}_Clean{ext}")
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(cleaned_data, f, ensure_ascii=False, indent=4)

    print(f"✅ Cleaned {input_file} saved to {output_path}")

"""
Combine Talent Table 
"""
def clean_and_combine_talent_table():
    with open("../1_Dirty/TalentTableChinese.json", "r", encoding="utf-8") as f:
        data_chinese = json.load(f)
    with open("../1_Dirty/TalentTableEnglish.json", "r", encoding="utf-8") as f:
        data_english = json.load(f)
    talent_data = {}
    for key in data_chinese:
        talent_data[key] = {}
        if data_chinese.get(key, {}).get("TalentName$chinese"):
            talent_data[key]["TalentName$chinese"] = data_chinese[key]["TalentName$chinese"]
        if data_chinese.get(key, {}).get("TalentDes$chinese"):
            talent_data[key]["TalentDes$chinese"] = data_chinese[key]["TalentDes$chinese"]
        if data_english.get(key, {}).get("TalentName$english"):
            talent_data[key]["TalentName$english"] = data_english[key]["TalentName$english"]
        if data_english.get(key, {}).get("TalentDes$english"):
            talent_data[key]["TalentDes$english"] = data_english[key]["TalentDes$english"]
    OUTPUT_FOLDER = "../4_Final/TalentTable_Clean.json"
    with open(OUTPUT_FOLDER, "w", encoding="utf-8") as f:
        json.dump(talent_data, f, ensure_ascii=False, indent=4)
    print(f"✅ Cleaned TalentTable saved to {OUTPUT_FOLDER}")



def combine_clean_tables():
    INPUT_FOLDER = "../2_Clean"
    OUTPUT_FILE = "../3_Combined/Combined.json"
    combined = {}
    # Iterate over all JSON files in the folder
    for filename in os.listdir(INPUT_FOLDER):
        if not filename.endswith(".json"):
            continue

        filepath = os.path.join(INPUT_FOLDER, filename)

        with open(filepath, "r", encoding="utf-8") as f:
            data = json.load(f)

        # Merge by ID
        for key, value in data.items():
            if key not in combined:
                combined[key] = {}
            combined[key][filename] = value
    combined_sorted = OrderedDict(sorted(combined.items(), key=lambda x: int(x[0])))
    # Write combined result
    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        json.dump(combined_sorted, f, ensure_ascii=False, indent=4)
    print(f"✅ Combined JSON saved to {OUTPUT_FILE}")

if __name__ == "__main__":
    clean_buff_table("../1_Dirty/", "BuffTable.json")
    clean_recount_table("../1_Dirty/", "RecountTable.json")
    clean_skill_names("../1_Dirty/", "skill_names.json")
    clean_buff_table("../1_Dirty/", "SkillTable.json")
    clean_and_combine_talent_table()
    combine_clean_tables()


