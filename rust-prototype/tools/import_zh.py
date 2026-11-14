"""Build the local Simplified Chinese catalog from a PretFRLG checkout."""
import argparse
import collections
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOKEN = re.compile(r"\{([^{}]+)\}")
HAN = re.compile(r"[\u3400-\u9fff]")
ALIASES = {"B_OPPONENT_MON1_NAME": "RIVAL", **{f"B_COPY_VAR_{i}": f"STR_VAR_{i}" for i in range(1, 4)}}
CONTROLS = {"PAUSE_UNTIL_PRESS", "PAUSE_MUSIC", "RESUME_MUSIC", "FONT_NORMAL", "FONT_MALE", "FONT_FEMALE"}
SYMBOLS = {"TRIANGLE": "▲", "CIRCLE_DOT": "●", "BIG_MULT_X": "×", "KUN": "", **{f"CIRCLE_{i}": str(i) for i in range(1, 10)}}
SCIENCE = "科学的力量真厉害！\\p现在可以用电脑存取道具和宝可梦了。$"


# Some Switch revision strings were removed or left in Japanese in the
# translation. Keep their original labels and supply just the missing copy.
OVERRIDES = {
    "PalletTown_RivalsHouse_Text_ThereYouGoAllDone": "{STR_VAR_1}露出了满足的表情……\p娜娜美：好啦，梳理好了。\n看，是不是漂亮多了？\p呵呵……真是可爱的宝可梦。$",
    "PewterCity_Museum_1F_Text_PleaseGoAround": "请从那边绕行。$",
    "PewterCity_Text_BerriesInsideUseCarefully": "里面装的是树果。\p有些树果很有用，\n要好好利用啊！$",
    "PewterCity_Text_DefeatedBrockYouCanHaveTreasure": "咦！你拿到了那个道馆徽章……\p真厉害，居然打赢了小刚！\p我太佩服你了。送你一件宝贝吧！$",
    "PewterCity_Text_MonsWillUseHeldBerriesOnTheirOwn": "让宝可梦携带树果，\n它们在对战中就会自己使用。\p比每次自己用伤药或解毒药更方便吧？$",
    "PewterCity_Text_OhPlayer": "哦，{PLAYER}！$",
    "CeruleanCity_BikeShop_Text_GermanFoldableBicyleFinallyOnMarket": "终于上市啦！\p德国制造的高级折叠自行车！$",
    "CeruleanCity_Text_RivalVictory": "毕竟，我可是个天才啊！$",
    "CeladonCity_Condominiums_RoofRoom_Text_BoxIsFull": "宝可梦太多了。\n先去换个盒子再来吧。$",
    "CeladonCity_GameCorner_PrizeRoom_Text_HereYouGo": "请收好。$",
    "CeladonCity_GameCorner_PrizeRoom_Text_OhFineThen": "啊，这样啊。$",
    "CeladonCity_GameCorner_PrizeRoom_Text_OopsNotEnoughRoom": "客人，你已经拿不下了。$",
    "FuchsiaCity_Text_Ellipsis": "……$",
    "FuchsiaCity_WardensHouse_Text_YouHaveTooMuchStuff": "哎呀，你的包包都装满啦！$",
    "LavenderTown_Mart_Text_SoldNuggetFromMountainsFor5000": "前些天，我在深山里捡到了一颗金珠！\p虽然没什么别的用处，\n竟然卖了 5000 元！$",
    "PokemonTower_2F_Text_RivalVictory": "{RIVAL}：哎呀，你的宝可梦还真倒下了！\n你得好好培育它们啊！$",
    "Route11_EastEntrance_1F_Text_BagIsFull": "{PLAYER}，你的包包好像装满了。$",
    "Route22_Text_LateRivalVictory": "{RIVAL}：哈哈，{PLAYER}！\n就这点本事可比不上我！\p再多练练吧，哈哈哈！$",
    "Route22_Text_RivalShouldCatchSomeMons": "{RIVAL}：想知道我为什么有两只宝可梦？\p你也去抓一只不就好了！$",
    "Route5_PokemonDayCare_Text_CantAcceptMonWithHM": "不好意思，学会秘传招式的宝可梦不能寄放在这里。$",
    "Route5_PokemonDayCare_Text_WantMeToTakeALookAtYours": "要我看看你的宝可梦吗？$",
    "SSAnne_2F_Corridor_Text_RivalVictory": "{PLAYER}……你晕船了吗？\p还是多锻炼锻炼吧！$",
    "SafariZone_North_Text_Fisherman": "我是来找丰缘地区没有的水属性宝可梦的……\p你知道湖在哪里吗？$",
    "SafariZone_North_Text_Man": "我要在这里抓许多稀有宝可梦，跟朋友交换！$",
    "VermilionCity_Gym_Text_AnotherSwitchInTrash": "翻找翻找……\n哎！垃圾桶底下还有一个开关！\p按下去试试吧。咔哒！$",
}


def read_texts(source, *, original=False):
    translated = {}
    for path in sorted((source / "data").rglob("*")):
        if path.suffix not in {".inc", ".s"}:
            continue
        name, parts = None, []
        enabled, stack = True, []
        for line in path.read_text(encoding="utf-8").splitlines():
            if m := re.fullmatch(r"\s*\.if(n?)def\s+(FIRERED|LEAFGREEN)\s*", line):
                match = (m[2] == "FIRERED") != bool(m[1])
                stack.append((enabled, match))
                enabled = enabled and match
            elif re.match(r"\s*\.if\b|\s*\.ifn?def\b", line):
                stack.append((enabled, True))
            elif re.match(r"\s*\.else\b", line) and stack:
                parent, branch = stack[-1]
                enabled = parent and not branch
            elif re.match(r"\s*\.endif\b", line) and stack:
                enabled, _ = stack.pop()
            elif enabled:
                if m := re.fullmatch(r"\s*(\w+)::?\s*", line):
                    if name and parts:
                        translated[name] = "".join(parts)
                    name, parts = m[1], []
                elif m := re.match(r'\s*\.(?:string|braille) "(.*)"\s*$', line):
                    parts.append(m[1])
        if name and parts:
            translated[name] = "".join(parts)
    if not original:
        translated["PalletTown_Text_CanStoreItemsAndMonsInPC"] = SCIENCE
    return translated


def clean(translated, original):
    def replace(match):
        key = match[1]
        if key in CONTROLS or key.startswith(("PLAY_BGM ", "PLAY_SE ", "PAUSE ", "CLEAR_TO ")):
            return ""
        if key in SYMBOLS:
            return SYMBOLS[key]
        return "{" + (key if key in original else ALIASES.get(key, key)) + "}"

    translated = TOKEN.sub(replace, translated)
    if set(TOKEN.findall(translated)) - set(TOKEN.findall(original)):
        return None
    if (set(TOKEN.findall(original)) & {"PLAYER", "RIVAL", "STR_VAR_1", "STR_VAR_2", "STR_VAR_3"}) - set(TOKEN.findall(translated)):
        return None
    return translated if HAN.search(translated) or translated == "……$" else None


def pages(text):
    return [page.replace(r"\n", "\n").replace(r"\l", "\n").replace(r'\"', '"')
            for page in text.rstrip("$").split(r"\p")]


def fixture_key(page):
    return " ".join(page.replace("Pokémon", "Pokemon").replace("POKéMON", "POKEMON").replace("POKéDEX", "POKEDEX").split())


def map_name(name):
    exact = {
        "Pallet Town": "真新镇", "Your home - downstairs": "小赤家一楼", "Your bedroom": "小赤家二楼",
        "Rival's house": "小绿家", "Oak's laboratory": "大木博士的研究所",
        "Burgled House": "被盗的民宅", "Bill's Sea Cottage": "正辉的海边小屋",
        "Volunteer Pokémon House": "宝可梦义工之家", "Power Plant": "无人发电厂",
        "Pokémon Fan Club": "宝可梦发烧友俱乐部", "Safari Zone Warden's House": "狩猎地带园长家",
    }
    if name in exact:
        return exact[name]
    parts = (
        ("Viridian Forest", "常青森林"), ("Viridian Pokémon Academy", "常青市宝可梦学校"),
        ("S.S. Anne", "圣特安努号"), ("Mt. Moon", "月见山"), ("Rock Tunnel", "岩山隧道"),
        ("Pokémon Tower", "宝可梦塔"), ("Rocket Hideout", "火箭队基地"),
        ("Underground Path", "地下通道"), ("Diglett's Cave", "地鼠洞穴"),
        ("Safari Zone", "狩猎地带"), ("Viridian", "常青市"), ("Pewter", "深灰市"),
        ("Cerulean", "华蓝市"), ("Vermilion", "枯叶市"), ("Lavender", "紫苑镇"),
        ("Celadon", "玉虹市"), ("Fuchsia", "浅红市"), ("Pokémon Center", "宝可梦中心"),
        ("Poké Mart", "友好商店"), ("Dept. Store", "百货公司"), ("Game Corner", "游戏城"),
        ("Prize Room", "奖品兑换处"), ("Fishing House", "钓鱼之家"), ("Day Care", "培育屋"),
        ("Bike Shop", "自行车店"), ("Rest House", "休息站"), ("Secret House", "秘密小屋"),
        ("Captain's Office", "船长室"), ("(East-West)", "（东西向）"),
        ("Gym", "道馆"), ("Museum", "科学博物馆"), ("Mansion", "大厦"),
        ("Hotel", "旅馆"), ("Restaurant", "餐厅"), ("Roof Room", "屋顶小屋"),
        ("Roof", "屋顶"), ("Elevator", "电梯"), ("Exterior", "外部"), ("Deck", "甲板"),
        ("Kitchen", "厨房"), ("Corridor", "走廊"), ("Cabin", "客舱"),
        ("North Entrance", "北入口"), ("South Entrance", "南入口"),
        ("East Entrance", "东入口"), ("West Entrance", "西入口"), ("Entrance", "入口"),
        ("North Gate", "北关卡"), ("South Gate", "南关卡"), ("East Gate", "东关卡"),
        ("West Gate", "西关卡"), ("Gate", "关卡"), ("Office", "管理处"),
        ("Center", "中央区"), ("North", "北区"), ("East", "东区"), ("West", "西区"),
    )
    name = re.sub(r"Route (\d+)", r"\1号道路", name)
    name = re.sub(r"\b(B?)(\d+)F\b", lambda m: ("地下" if m[1] else "") + m[2] + "楼", name)
    for old, new in parts:
        name = name.replace(old, new)
    return re.sub(r" ?(?:City|Town)\b", "", name).replace("House", "民宅").replace("house", "民宅").replace(" ", "")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    args = parser.parse_args()
    source = args.source
    field = json.loads((ROOT / "data/field.json").read_text())
    translations = read_texts(source)
    translations.update(OVERRIDES)
    texts, fixtures = {}, {}
    matches = {}
    for label, address in field["labels"].items():
        original = field["texts"].get(str(address))
        if original is None or label not in translations:
            continue
        translated = clean(translations[label], original)
        if translated:
            texts[address] = translated
            matches[label] = (original, translated)

    # Match the offline exploration fixture to the same script labels as the VM.
    # There are a few fixture-only text labels outside the assembled script data.
    original_sources = read_texts(ROOT / "reference/pokefirered", original=True)
    for label, original in original_sources.items():
        if label in translations and label not in matches:
            translated = clean(translations[label], original)
            if translated:
                matches[label] = (original, translated)
    for original, translated in matches.values():
        old = pages(original.replace("{PLAYER}", "RED").replace("{RIVAL}", "BLUE"))
        new = pages(translated.replace("{PLAYER}", "小赤").replace("{RIVAL}", "小绿"))
        if TOKEN.search("".join(new)):
            continue
        # Keep the fixture's existing page count, distributing translated pages
        # in order when Chinese typesetting changes the number of pages.
        for i, page in enumerate(old):
            start = i * len(new) // len(old)
            end = (i + 1) * len(new) // len(old)
            if start != end:
                fixtures.setdefault(fixture_key(page), "\n\n".join(new[start:end]))

    # The lightweight exploration fixture does not execute the original
    # script that fills these variables, so use self-contained prompts.
    fixtures.update({
        "There's an e-mail message here.": "电脑里有一封电子邮件。",
        "Hi! Do you have a {STR_VAR_1}?": "你好！你有我想找的宝可梦吗？",
        "Want to trade it for my {STR_VAR_2}?": "要和我交换宝可梦吗？",
        "If your POKEDEX has complete data on {STR_VAR_1} species, I'm supposed to give you a reward.": "如果你收集了足够多的图鉴资料，我会给你一份奖励。",
        "PROF. OAK entrusted me with the {STR_VAR_2} for you.": "大木博士托我把这份道具交给你。",
        "Have you gathered data on at least {STR_VAR_1} kinds of POKEMON?": "你收集到足够多的宝可梦图鉴资料了吗？",
    })
    species_text = (source / "src/data/text/species_names.h").read_text()
    pairs = collections.defaultdict(list)
    for id, name in re.findall(r'\[(SPECIES_\w+)\]\s*=\s*_\("(.*?)"\)', species_text):
        pairs[id].append(name)
    species = {names[0].upper(): names[-1] for names in pairs.values() if HAN.search(names[-1])}
    moves = dict(re.findall(r'\[(MOVE_\w+)\]\s*=\s*_\("(.*?)"\)', (source / "src/data/text/move_names.h").read_text()))
    items_src = json.loads((source / "src/data/items.json").read_text())
    item_names = {item["itemId"]: item["english"] for item in items_src["items"] if HAN.search(item["english"])}
    originals = json.loads((ROOT / "reference/pokefirered/src/data/items.json").read_text())
    items = {item["english"].upper(): item_names[item["itemId"]] for item in originals["items"] if item["itemId"] in item_names}
    world = json.loads((ROOT / "data/world.json").read_text())
    maps = {m["id"]: map_name(m["name"]) for m in world["maps"]}
    untranslated = [name for name in maps.values() if re.search(r"[A-Za-z]", name)]
    if untranslated:
        raise ValueError(f"Untranslated map names: {untranslated}")
    output = ROOT / "assets/locale/zh-CN.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(dict(texts=texts, fixtures=fixtures, species=species, moves=moves, items=items, maps=maps), ensure_ascii=False), encoding="utf-8")
    print(f"Imported {len(texts)}/{len(field['texts'])} field texts, {len(fixtures)} fixture pages, {len(species)} species, {len(moves)} moves, {len(items)} items.")


if __name__ == "__main__":
    main()
