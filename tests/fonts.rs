use pace::fonts::{FONT_NAMES, Fonts};

// Captured with the original FIGlet.js 1.5.2 and pace-ts default layout options.
const ORIGINAL_SAMPLES: &[(&str, &str)] = &[
    (
        "1Row",
        include_str!("../assets/fonts/snapshots/ONE_ROW.txt"),
    ),
    ("3-D", include_str!("../assets/fonts/snapshots/THREE_D.txt")),
    (
        "3D Diagonal",
        include_str!("../assets/fonts/snapshots/THREED_DIAGONAL.txt"),
    ),
    (
        "3D-ASCII",
        include_str!("../assets/fonts/snapshots/THREED_ASCII.txt"),
    ),
    ("3x5", include_str!("../assets/fonts/snapshots/THREEX5.txt")),
    (
        "4Max",
        include_str!("../assets/fonts/snapshots/FOURMAX.txt"),
    ),
    (
        "5 Line Oblique",
        include_str!("../assets/fonts/snapshots/FIVE_LINE_OBLIQUE.txt"),
    ),
    (
        "AMC 3 Line",
        include_str!("../assets/fonts/snapshots/AMC_3_LINE.txt"),
    ),
    (
        "AMC 3 Liv1",
        include_str!("../assets/fonts/snapshots/AMC_3_LIV1.txt"),
    ),
    (
        "AMC AAA01",
        include_str!("../assets/fonts/snapshots/AMC_AAA01.txt"),
    ),
    (
        "AMC Neko",
        include_str!("../assets/fonts/snapshots/AMC_NEKO.txt"),
    ),
    (
        "AMC Razor",
        include_str!("../assets/fonts/snapshots/AMC_RAZOR.txt"),
    ),
    (
        "AMC Razor2",
        include_str!("../assets/fonts/snapshots/AMC_RAZOR2.txt"),
    ),
    (
        "AMC Slash",
        include_str!("../assets/fonts/snapshots/AMC_SLASH.txt"),
    ),
    (
        "AMC Slider",
        include_str!("../assets/fonts/snapshots/AMC_SLIDER.txt"),
    ),
    (
        "AMC Thin",
        include_str!("../assets/fonts/snapshots/AMC_THIN.txt"),
    ),
    (
        "AMC Tubes",
        include_str!("../assets/fonts/snapshots/AMC_TUBES.txt"),
    ),
    (
        "AMC Untitled",
        include_str!("../assets/fonts/snapshots/AMC_UNTITLED.txt"),
    ),
    (
        "ANSI Regular",
        include_str!("../assets/fonts/snapshots/ANSI_REGULAR.txt"),
    ),
    (
        "ANSI Shadow",
        include_str!("../assets/fonts/snapshots/ANSI_SHADOW.txt"),
    ),
    (
        "ASCII New Roman",
        include_str!("../assets/fonts/snapshots/ASCII_NEW_ROMAN.txt"),
    ),
    (
        "Acrobatic",
        include_str!("../assets/fonts/snapshots/ACROBATIC.txt"),
    ),
    (
        "Alligator",
        include_str!("../assets/fonts/snapshots/ALLIGATOR.txt"),
    ),
    (
        "Alligator2",
        include_str!("../assets/fonts/snapshots/ALLIGATOR2.txt"),
    ),
    ("Alpha", include_str!("../assets/fonts/snapshots/ALPHA.txt")),
    (
        "Alphabet",
        include_str!("../assets/fonts/snapshots/ALPHABET.txt"),
    ),
    (
        "Arrows",
        include_str!("../assets/fonts/snapshots/ARROWS.txt"),
    ),
    (
        "Avatar",
        include_str!("../assets/fonts/snapshots/AVATAR.txt"),
    ),
    ("B1FF", include_str!("../assets/fonts/snapshots/B1FF.txt")),
    (
        "Banner",
        include_str!("../assets/fonts/snapshots/BANNER.txt"),
    ),
    (
        "Banner3-D",
        include_str!("../assets/fonts/snapshots/BANNER3_D.txt"),
    ),
    (
        "Banner3",
        include_str!("../assets/fonts/snapshots/BANNER3.txt"),
    ),
    (
        "Banner4",
        include_str!("../assets/fonts/snapshots/BANNER4.txt"),
    ),
    (
        "Barbwire",
        include_str!("../assets/fonts/snapshots/BARBWIRE.txt"),
    ),
    ("Basic", include_str!("../assets/fonts/snapshots/BASIC.txt")),
    ("Bear", include_str!("../assets/fonts/snapshots/BEAR.txt")),
    ("Bell", include_str!("../assets/fonts/snapshots/BELL.txt")),
    (
        "Benjamin",
        include_str!("../assets/fonts/snapshots/BENJAMIN.txt"),
    ),
    (
        "Big Chief",
        include_str!("../assets/fonts/snapshots/BIG_CHIEF.txt"),
    ),
    (
        "Big Money-ne",
        include_str!("../assets/fonts/snapshots/BIG_MONEY_NE.txt"),
    ),
    (
        "Big Money-nw",
        include_str!("../assets/fonts/snapshots/BIG_MONEY_NW.txt"),
    ),
    (
        "Big Money-se",
        include_str!("../assets/fonts/snapshots/BIG_MONEY_SE.txt"),
    ),
    (
        "Big Money-sw",
        include_str!("../assets/fonts/snapshots/BIG_MONEY_SW.txt"),
    ),
    ("Big", include_str!("../assets/fonts/snapshots/BIG.txt")),
    (
        "Bigfig",
        include_str!("../assets/fonts/snapshots/BIGFIG.txt"),
    ),
    (
        "Binary",
        include_str!("../assets/fonts/snapshots/BINARY.txt"),
    ),
    ("Block", include_str!("../assets/fonts/snapshots/BLOCK.txt")),
    (
        "Blocks",
        include_str!("../assets/fonts/snapshots/BLOCKS.txt"),
    ),
    (
        "Bloody",
        include_str!("../assets/fonts/snapshots/BLOODY.txt"),
    ),
    (
        "Bolger",
        include_str!("../assets/fonts/snapshots/BOLGER.txt"),
    ),
    (
        "Braced",
        include_str!("../assets/fonts/snapshots/BRACED.txt"),
    ),
    (
        "Bright",
        include_str!("../assets/fonts/snapshots/BRIGHT.txt"),
    ),
    (
        "Broadway KB",
        include_str!("../assets/fonts/snapshots/BROADWAY_KB.txt"),
    ),
    (
        "Broadway",
        include_str!("../assets/fonts/snapshots/BROADWAY.txt"),
    ),
    (
        "Bubble",
        include_str!("../assets/fonts/snapshots/BUBBLE.txt"),
    ),
    (
        "Bulbhead",
        include_str!("../assets/fonts/snapshots/BULBHEAD.txt"),
    ),
    (
        "Caligraphy",
        include_str!("../assets/fonts/snapshots/CALIGRAPHY.txt"),
    ),
    (
        "Caligraphy2",
        include_str!("../assets/fonts/snapshots/CALIGRAPHY2.txt"),
    ),
    (
        "Calvin S",
        include_str!("../assets/fonts/snapshots/CALVIN_S.txt"),
    ),
    ("Cards", include_str!("../assets/fonts/snapshots/CARDS.txt")),
    (
        "Catwalk",
        include_str!("../assets/fonts/snapshots/CATWALK.txt"),
    ),
    (
        "Chiseled",
        include_str!("../assets/fonts/snapshots/CHISELED.txt"),
    ),
    (
        "Chunky",
        include_str!("../assets/fonts/snapshots/CHUNKY.txt"),
    ),
    (
        "Coinstak",
        include_str!("../assets/fonts/snapshots/COINSTAK.txt"),
    ),
    ("Cola", include_str!("../assets/fonts/snapshots/COLA.txt")),
    (
        "Colossal",
        include_str!("../assets/fonts/snapshots/COLOSSAL.txt"),
    ),
    (
        "Computer",
        include_str!("../assets/fonts/snapshots/COMPUTER.txt"),
    ),
    (
        "Contessa",
        include_str!("../assets/fonts/snapshots/CONTESSA.txt"),
    ),
    (
        "Contrast",
        include_str!("../assets/fonts/snapshots/CONTRAST.txt"),
    ),
    (
        "Cosmike",
        include_str!("../assets/fonts/snapshots/COSMIKE.txt"),
    ),
    (
        "Crawford",
        include_str!("../assets/fonts/snapshots/CRAWFORD.txt"),
    ),
    (
        "Crawford2",
        include_str!("../assets/fonts/snapshots/CRAWFORD2.txt"),
    ),
    ("Crazy", include_str!("../assets/fonts/snapshots/CRAZY.txt")),
    (
        "Cricket",
        include_str!("../assets/fonts/snapshots/CRICKET.txt"),
    ),
    (
        "Cursive",
        include_str!("../assets/fonts/snapshots/CURSIVE.txt"),
    ),
    (
        "Cyberlarge",
        include_str!("../assets/fonts/snapshots/CYBERLARGE.txt"),
    ),
    (
        "Cybermedium",
        include_str!("../assets/fonts/snapshots/CYBERMEDIUM.txt"),
    ),
    (
        "Cybersmall",
        include_str!("../assets/fonts/snapshots/CYBERSMALL.txt"),
    ),
    (
        "Cygnet",
        include_str!("../assets/fonts/snapshots/CYGNET.txt"),
    ),
    ("DANC4", include_str!("../assets/fonts/snapshots/DANC4.txt")),
    (
        "DOS Rebel",
        include_str!("../assets/fonts/snapshots/DOS_REBEL.txt"),
    ),
    (
        "DWhistled",
        include_str!("../assets/fonts/snapshots/DWHISTLED.txt"),
    ),
    (
        "Dancing Font",
        include_str!("../assets/fonts/snapshots/DANCING_FONT.txt"),
    ),
    (
        "Decimal",
        include_str!("../assets/fonts/snapshots/DECIMAL.txt"),
    ),
    (
        "Def Leppard",
        include_str!("../assets/fonts/snapshots/DEF_LEPPARD.txt"),
    ),
    (
        "Delta Corps Priest 1",
        include_str!("../assets/fonts/snapshots/DELTA_CORPS_PRIEST_1.txt"),
    ),
    (
        "Diamond",
        include_str!("../assets/fonts/snapshots/DIAMOND.txt"),
    ),
    (
        "Diet Cola",
        include_str!("../assets/fonts/snapshots/DIET_COLA.txt"),
    ),
    (
        "Digital",
        include_str!("../assets/fonts/snapshots/DIGITAL.txt"),
    ),
    ("Doh", include_str!("../assets/fonts/snapshots/DOH.txt")),
    ("Doom", include_str!("../assets/fonts/snapshots/DOOM.txt")),
    (
        "Dot Matrix",
        include_str!("../assets/fonts/snapshots/DOT_MATRIX.txt"),
    ),
    (
        "Double Shorts",
        include_str!("../assets/fonts/snapshots/DOUBLE_SHORTS.txt"),
    ),
    (
        "Double",
        include_str!("../assets/fonts/snapshots/DOUBLE.txt"),
    ),
    (
        "Dr Pepper",
        include_str!("../assets/fonts/snapshots/DR_PEPPER.txt"),
    ),
    (
        "Efti Chess",
        include_str!("../assets/fonts/snapshots/EFTI_CHESS.txt"),
    ),
    (
        "Efti Font",
        include_str!("../assets/fonts/snapshots/EFTI_FONT.txt"),
    ),
    (
        "Efti Italic",
        include_str!("../assets/fonts/snapshots/EFTI_ITALIC.txt"),
    ),
    (
        "Efti Piti",
        include_str!("../assets/fonts/snapshots/EFTI_PITI.txt"),
    ),
    (
        "Efti Robot",
        include_str!("../assets/fonts/snapshots/EFTI_ROBOT.txt"),
    ),
    (
        "Efti Wall",
        include_str!("../assets/fonts/snapshots/EFTI_WALL.txt"),
    ),
    (
        "Efti Water",
        include_str!("../assets/fonts/snapshots/EFTI_WATER.txt"),
    ),
    (
        "Electronic",
        include_str!("../assets/fonts/snapshots/ELECTRONIC.txt"),
    ),
    ("Elite", include_str!("../assets/fonts/snapshots/ELITE.txt")),
    ("Epic", include_str!("../assets/fonts/snapshots/EPIC.txt")),
    (
        "Fender",
        include_str!("../assets/fonts/snapshots/FENDER.txt"),
    ),
    (
        "Filter",
        include_str!("../assets/fonts/snapshots/FILTER.txt"),
    ),
    (
        "Fire Font-k",
        include_str!("../assets/fonts/snapshots/FIRE_FONT_K.txt"),
    ),
    (
        "Fire Font-s",
        include_str!("../assets/fonts/snapshots/FIRE_FONT_S.txt"),
    ),
    (
        "Flipped",
        include_str!("../assets/fonts/snapshots/FLIPPED.txt"),
    ),
    (
        "Flower Power",
        include_str!("../assets/fonts/snapshots/FLOWER_POWER.txt"),
    ),
    (
        "Four Tops",
        include_str!("../assets/fonts/snapshots/FOUR_TOPS.txt"),
    ),
    (
        "Fraktur",
        include_str!("../assets/fonts/snapshots/FRAKTUR.txt"),
    ),
    (
        "Fun Face",
        include_str!("../assets/fonts/snapshots/FUN_FACE.txt"),
    ),
    (
        "Fun Faces",
        include_str!("../assets/fonts/snapshots/FUN_FACES.txt"),
    ),
    ("Fuzzy", include_str!("../assets/fonts/snapshots/FUZZY.txt")),
    (
        "Georgi16",
        include_str!("../assets/fonts/snapshots/GEORGI16.txt"),
    ),
    (
        "Georgia11",
        include_str!("../assets/fonts/snapshots/GEORGIA11.txt"),
    ),
    ("Ghost", include_str!("../assets/fonts/snapshots/GHOST.txt")),
    (
        "Ghoulish",
        include_str!("../assets/fonts/snapshots/GHOULISH.txt"),
    ),
    (
        "Glenyn",
        include_str!("../assets/fonts/snapshots/GLENYN.txt"),
    ),
    ("Goofy", include_str!("../assets/fonts/snapshots/GOOFY.txt")),
    (
        "Gothic",
        include_str!("../assets/fonts/snapshots/GOTHIC.txt"),
    ),
    (
        "Graceful",
        include_str!("../assets/fonts/snapshots/GRACEFUL.txt"),
    ),
    (
        "Gradient",
        include_str!("../assets/fonts/snapshots/GRADIENT.txt"),
    ),
    (
        "Graffiti",
        include_str!("../assets/fonts/snapshots/GRAFFITI.txt"),
    ),
    ("Greek", include_str!("../assets/fonts/snapshots/GREEK.txt")),
    (
        "Heart Left",
        include_str!("../assets/fonts/snapshots/HEART_LEFT.txt"),
    ),
    (
        "Heart Right",
        include_str!("../assets/fonts/snapshots/HEART_RIGHT.txt"),
    ),
    (
        "Henry 3D",
        include_str!("../assets/fonts/snapshots/HENRY_3D.txt"),
    ),
    ("Hex", include_str!("../assets/fonts/snapshots/HEX.txt")),
    (
        "Hieroglyphs",
        include_str!("../assets/fonts/snapshots/HIEROGLYPHS.txt"),
    ),
    (
        "Hollywood",
        include_str!("../assets/fonts/snapshots/HOLLYWOOD.txt"),
    ),
    (
        "Horizontal Left",
        include_str!("../assets/fonts/snapshots/HORIZONTAL_LEFT.txt"),
    ),
    (
        "Horizontal Right",
        include_str!("../assets/fonts/snapshots/HORIZONTAL_RIGHT.txt"),
    ),
    (
        "ICL-1900",
        include_str!("../assets/fonts/snapshots/ICL_1900.txt"),
    ),
    (
        "Impossible",
        include_str!("../assets/fonts/snapshots/IMPOSSIBLE.txt"),
    ),
    (
        "Invita",
        include_str!("../assets/fonts/snapshots/INVITA.txt"),
    ),
    (
        "Isometric1",
        include_str!("../assets/fonts/snapshots/ISOMETRIC1.txt"),
    ),
    (
        "Isometric2",
        include_str!("../assets/fonts/snapshots/ISOMETRIC2.txt"),
    ),
    (
        "Isometric3",
        include_str!("../assets/fonts/snapshots/ISOMETRIC3.txt"),
    ),
    (
        "Isometric4",
        include_str!("../assets/fonts/snapshots/ISOMETRIC4.txt"),
    ),
    (
        "Italic",
        include_str!("../assets/fonts/snapshots/ITALIC.txt"),
    ),
    ("Ivrit", include_str!("../assets/fonts/snapshots/IVRIT.txt")),
    (
        "JS Block Letters",
        include_str!("../assets/fonts/snapshots/JS_BLOCK_LETTERS.txt"),
    ),
    (
        "JS Bracket Letters",
        include_str!("../assets/fonts/snapshots/JS_BRACKET_LETTERS.txt"),
    ),
    (
        "JS Capital Curves",
        include_str!("../assets/fonts/snapshots/JS_CAPITAL_CURVES.txt"),
    ),
    (
        "JS Cursive",
        include_str!("../assets/fonts/snapshots/JS_CURSIVE.txt"),
    ),
    (
        "JS Stick Letters",
        include_str!("../assets/fonts/snapshots/JS_STICK_LETTERS.txt"),
    ),
    ("Jacky", include_str!("../assets/fonts/snapshots/JACKY.txt")),
    (
        "Jazmine",
        include_str!("../assets/fonts/snapshots/JAZMINE.txt"),
    ),
    (
        "Jerusalem",
        include_str!("../assets/fonts/snapshots/JERUSALEM.txt"),
    ),
    (
        "Katakana",
        include_str!("../assets/fonts/snapshots/KATAKANA.txt"),
    ),
    ("Kban", include_str!("../assets/fonts/snapshots/KBAN.txt")),
    (
        "Keyboard",
        include_str!("../assets/fonts/snapshots/KEYBOARD.txt"),
    ),
    ("Knob", include_str!("../assets/fonts/snapshots/KNOB.txt")),
    (
        "Konto Slant",
        include_str!("../assets/fonts/snapshots/KONTO_SLANT.txt"),
    ),
    ("Konto", include_str!("../assets/fonts/snapshots/KONTO.txt")),
    ("LCD", include_str!("../assets/fonts/snapshots/LCD.txt")),
    (
        "Larry 3D 2",
        include_str!("../assets/fonts/snapshots/LARRY_3D_2.txt"),
    ),
    (
        "Larry 3D",
        include_str!("../assets/fonts/snapshots/LARRY_3D.txt"),
    ),
    ("Lean", include_str!("../assets/fonts/snapshots/LEAN.txt")),
    (
        "Letters",
        include_str!("../assets/fonts/snapshots/LETTERS.txt"),
    ),
    (
        "Lil Devil",
        include_str!("../assets/fonts/snapshots/LIL_DEVIL.txt"),
    ),
    (
        "Line Blocks",
        include_str!("../assets/fonts/snapshots/LINE_BLOCKS.txt"),
    ),
    ("Linux", include_str!("../assets/fonts/snapshots/LINUX.txt")),
    (
        "Lockergnome",
        include_str!("../assets/fonts/snapshots/LOCKERGNOME.txt"),
    ),
    (
        "Madrid",
        include_str!("../assets/fonts/snapshots/MADRID.txt"),
    ),
    (
        "Marquee",
        include_str!("../assets/fonts/snapshots/MARQUEE.txt"),
    ),
    (
        "Maxfour",
        include_str!("../assets/fonts/snapshots/MAXFOUR.txt"),
    ),
    (
        "Merlin1",
        include_str!("../assets/fonts/snapshots/MERLIN1.txt"),
    ),
    (
        "Merlin2",
        include_str!("../assets/fonts/snapshots/MERLIN2.txt"),
    ),
    ("Mike", include_str!("../assets/fonts/snapshots/MIKE.txt")),
    ("Mini", include_str!("../assets/fonts/snapshots/MINI.txt")),
    (
        "Mirror",
        include_str!("../assets/fonts/snapshots/MIRROR.txt"),
    ),
    (
        "Mnemonic",
        include_str!("../assets/fonts/snapshots/MNEMONIC.txt"),
    ),
    (
        "Modular",
        include_str!("../assets/fonts/snapshots/MODULAR.txt"),
    ),
    ("Morse", include_str!("../assets/fonts/snapshots/MORSE.txt")),
    (
        "Morse2",
        include_str!("../assets/fonts/snapshots/MORSE2.txt"),
    ),
    (
        "Moscow",
        include_str!("../assets/fonts/snapshots/MOSCOW.txt"),
    ),
    (
        "Mshebrew210",
        include_str!("../assets/fonts/snapshots/MSHEBREW210.txt"),
    ),
    (
        "Muzzle",
        include_str!("../assets/fonts/snapshots/MUZZLE.txt"),
    ),
    (
        "NScript",
        include_str!("../assets/fonts/snapshots/NSCRIPT.txt"),
    ),
    (
        "NT Greek",
        include_str!("../assets/fonts/snapshots/NT_GREEK.txt"),
    ),
    (
        "NV Script",
        include_str!("../assets/fonts/snapshots/NV_SCRIPT.txt"),
    ),
    (
        "Nancyj-Fancy",
        include_str!("../assets/fonts/snapshots/NANCYJ_FANCY.txt"),
    ),
    (
        "Nancyj-Improved",
        include_str!("../assets/fonts/snapshots/NANCYJ_IMPROVED.txt"),
    ),
    (
        "Nancyj-Underlined",
        include_str!("../assets/fonts/snapshots/NANCYJ_UNDERLINED.txt"),
    ),
    (
        "Nancyj",
        include_str!("../assets/fonts/snapshots/NANCYJ.txt"),
    ),
    (
        "Nipples",
        include_str!("../assets/fonts/snapshots/NIPPLES.txt"),
    ),
    ("O8", include_str!("../assets/fonts/snapshots/O8.txt")),
    ("OS2", include_str!("../assets/fonts/snapshots/OS2.txt")),
    ("Octal", include_str!("../assets/fonts/snapshots/OCTAL.txt")),
    ("Ogre", include_str!("../assets/fonts/snapshots/OGRE.txt")),
    (
        "Old Banner",
        include_str!("../assets/fonts/snapshots/OLD_BANNER.txt"),
    ),
    ("Pagga", include_str!("../assets/fonts/snapshots/PAGGA.txt")),
    (
        "Patorjk's Cheese",
        include_str!("../assets/fonts/snapshots/PATORJKS_CHEESE.txt"),
    ),
    (
        "Patorjk-HeX",
        include_str!("../assets/fonts/snapshots/PATORJK_HEX.txt"),
    ),
    ("Pawp", include_str!("../assets/fonts/snapshots/PAWP.txt")),
    (
        "Peaks Slant",
        include_str!("../assets/fonts/snapshots/PEAKS_SLANT.txt"),
    ),
    ("Peaks", include_str!("../assets/fonts/snapshots/PEAKS.txt")),
    (
        "Pebbles",
        include_str!("../assets/fonts/snapshots/PEBBLES.txt"),
    ),
    (
        "Pepper",
        include_str!("../assets/fonts/snapshots/PEPPER.txt"),
    ),
    (
        "Poison",
        include_str!("../assets/fonts/snapshots/POISON.txt"),
    ),
    ("Puffy", include_str!("../assets/fonts/snapshots/PUFFY.txt")),
    (
        "Puzzle",
        include_str!("../assets/fonts/snapshots/PUZZLE.txt"),
    ),
    (
        "Pyramid",
        include_str!("../assets/fonts/snapshots/PYRAMID.txt"),
    ),
    (
        "Rammstein",
        include_str!("../assets/fonts/snapshots/RAMMSTEIN.txt"),
    ),
    (
        "Rectangles",
        include_str!("../assets/fonts/snapshots/RECTANGLES.txt"),
    ),
    (
        "Red Phoenix",
        include_str!("../assets/fonts/snapshots/RED_PHOENIX.txt"),
    ),
    (
        "Relief",
        include_str!("../assets/fonts/snapshots/RELIEF.txt"),
    ),
    (
        "Relief2",
        include_str!("../assets/fonts/snapshots/RELIEF2.txt"),
    ),
    (
        "Reverse",
        include_str!("../assets/fonts/snapshots/REVERSE.txt"),
    ),
    ("Roman", include_str!("../assets/fonts/snapshots/ROMAN.txt")),
    ("Rot13", include_str!("../assets/fonts/snapshots/ROT13.txt")),
    (
        "Rotated",
        include_str!("../assets/fonts/snapshots/ROTATED.txt"),
    ),
    (
        "Rounded",
        include_str!("../assets/fonts/snapshots/ROUNDED.txt"),
    ),
    (
        "Rowan Cap",
        include_str!("../assets/fonts/snapshots/ROWAN_CAP.txt"),
    ),
    ("Rozzo", include_str!("../assets/fonts/snapshots/ROZZO.txt")),
    ("Runic", include_str!("../assets/fonts/snapshots/RUNIC.txt")),
    ("Runyc", include_str!("../assets/fonts/snapshots/RUNYC.txt")),
    (
        "S Blood",
        include_str!("../assets/fonts/snapshots/S_BLOOD.txt"),
    ),
    (
        "SL Script",
        include_str!("../assets/fonts/snapshots/SL_SCRIPT.txt"),
    ),
    (
        "Santa Clara",
        include_str!("../assets/fonts/snapshots/SANTA_CLARA.txt"),
    ),
    (
        "Script",
        include_str!("../assets/fonts/snapshots/SCRIPT.txt"),
    ),
    (
        "Serifcap",
        include_str!("../assets/fonts/snapshots/SERIFCAP.txt"),
    ),
    (
        "Shadow",
        include_str!("../assets/fonts/snapshots/SHADOW.txt"),
    ),
    (
        "Shimrod",
        include_str!("../assets/fonts/snapshots/SHIMROD.txt"),
    ),
    ("Short", include_str!("../assets/fonts/snapshots/SHORT.txt")),
    (
        "Slant Relief",
        include_str!("../assets/fonts/snapshots/SLANT_RELIEF.txt"),
    ),
    ("Slant", include_str!("../assets/fonts/snapshots/SLANT.txt")),
    ("Slide", include_str!("../assets/fonts/snapshots/SLIDE.txt")),
    (
        "Small Caps",
        include_str!("../assets/fonts/snapshots/SMALL_CAPS.txt"),
    ),
    (
        "Small Isometric1",
        include_str!("../assets/fonts/snapshots/SMALL_ISOMETRIC1.txt"),
    ),
    (
        "Small Keyboard",
        include_str!("../assets/fonts/snapshots/SMALL_KEYBOARD.txt"),
    ),
    (
        "Small Poison",
        include_str!("../assets/fonts/snapshots/SMALL_POISON.txt"),
    ),
    (
        "Small Script",
        include_str!("../assets/fonts/snapshots/SMALL_SCRIPT.txt"),
    ),
    (
        "Small Shadow",
        include_str!("../assets/fonts/snapshots/SMALL_SHADOW.txt"),
    ),
    (
        "Small Slant",
        include_str!("../assets/fonts/snapshots/SMALL_SLANT.txt"),
    ),
    (
        "Small Tengwar",
        include_str!("../assets/fonts/snapshots/SMALL_TENGWAR.txt"),
    ),
    ("Small", include_str!("../assets/fonts/snapshots/SMALL.txt")),
    ("Soft", include_str!("../assets/fonts/snapshots/SOFT.txt")),
    ("Speed", include_str!("../assets/fonts/snapshots/SPEED.txt")),
    (
        "Spliff",
        include_str!("../assets/fonts/snapshots/SPLIFF.txt"),
    ),
    (
        "Stacey",
        include_str!("../assets/fonts/snapshots/STACEY.txt"),
    ),
    (
        "Stampate",
        include_str!("../assets/fonts/snapshots/STAMPATE.txt"),
    ),
    (
        "Stampatello",
        include_str!("../assets/fonts/snapshots/STAMPATELLO.txt"),
    ),
    (
        "Standard",
        include_str!("../assets/fonts/snapshots/STANDARD.txt"),
    ),
    (
        "Star Strips",
        include_str!("../assets/fonts/snapshots/STAR_STRIPS.txt"),
    ),
    (
        "Star Wars",
        include_str!("../assets/fonts/snapshots/STAR_WARS.txt"),
    ),
    (
        "Stellar",
        include_str!("../assets/fonts/snapshots/STELLAR.txt"),
    ),
    (
        "Stforek",
        include_str!("../assets/fonts/snapshots/STFOREK.txt"),
    ),
    (
        "Stick Letters",
        include_str!("../assets/fonts/snapshots/STICK_LETTERS.txt"),
    ),
    ("Stop", include_str!("../assets/fonts/snapshots/STOP.txt")),
    (
        "Straight",
        include_str!("../assets/fonts/snapshots/STRAIGHT.txt"),
    ),
    (
        "Stronger Than All",
        include_str!("../assets/fonts/snapshots/STRONGER_THAN_ALL.txt"),
    ),
    (
        "Sub-Zero",
        include_str!("../assets/fonts/snapshots/SUB_ZERO.txt"),
    ),
    (
        "Swamp Land",
        include_str!("../assets/fonts/snapshots/SWAMP_LAND.txt"),
    ),
    ("Swan", include_str!("../assets/fonts/snapshots/SWAN.txt")),
    ("Sweet", include_str!("../assets/fonts/snapshots/SWEET.txt")),
    ("THIS", include_str!("../assets/fonts/snapshots/THIS.txt")),
    ("Tanja", include_str!("../assets/fonts/snapshots/TANJA.txt")),
    (
        "Tengwar",
        include_str!("../assets/fonts/snapshots/TENGWAR.txt"),
    ),
    ("Term", include_str!("../assets/fonts/snapshots/TERM.txt")),
    ("Test1", include_str!("../assets/fonts/snapshots/TEST1.txt")),
    (
        "The Edge",
        include_str!("../assets/fonts/snapshots/THE_EDGE.txt"),
    ),
    ("Thick", include_str!("../assets/fonts/snapshots/THICK.txt")),
    ("Thin", include_str!("../assets/fonts/snapshots/THIN.txt")),
    (
        "Thorned",
        include_str!("../assets/fonts/snapshots/THORNED.txt"),
    ),
    (
        "Three Point",
        include_str!("../assets/fonts/snapshots/THREE_POINT.txt"),
    ),
    (
        "Ticks Slant",
        include_str!("../assets/fonts/snapshots/TICKS_SLANT.txt"),
    ),
    ("Ticks", include_str!("../assets/fonts/snapshots/TICKS.txt")),
    ("Tiles", include_str!("../assets/fonts/snapshots/TILES.txt")),
    (
        "Tinker-Toy",
        include_str!("../assets/fonts/snapshots/TINKER_TOY.txt"),
    ),
    (
        "Tombstone",
        include_str!("../assets/fonts/snapshots/TOMBSTONE.txt"),
    ),
    ("Train", include_str!("../assets/fonts/snapshots/TRAIN.txt")),
    ("Trek", include_str!("../assets/fonts/snapshots/TREK.txt")),
    (
        "Tsalagi",
        include_str!("../assets/fonts/snapshots/TSALAGI.txt"),
    ),
    (
        "Tubular",
        include_str!("../assets/fonts/snapshots/TUBULAR.txt"),
    ),
    (
        "Twisted",
        include_str!("../assets/fonts/snapshots/TWISTED.txt"),
    ),
    (
        "Two Point",
        include_str!("../assets/fonts/snapshots/TWO_POINT.txt"),
    ),
    (
        "USA Flag",
        include_str!("../assets/fonts/snapshots/USA_FLAG.txt"),
    ),
    (
        "Univers",
        include_str!("../assets/fonts/snapshots/UNIVERS.txt"),
    ),
    (
        "Varsity",
        include_str!("../assets/fonts/snapshots/VARSITY.txt"),
    ),
    ("Wavy", include_str!("../assets/fonts/snapshots/WAVY.txt")),
    ("Weird", include_str!("../assets/fonts/snapshots/WEIRD.txt")),
    (
        "Wet Letter",
        include_str!("../assets/fonts/snapshots/WET_LETTER.txt"),
    ),
    (
        "Whimsy",
        include_str!("../assets/fonts/snapshots/WHIMSY.txt"),
    ),
    ("Wow", include_str!("../assets/fonts/snapshots/WOW.txt")),
];

#[test]
fn every_original_font_matches_the_original_sample() {
    assert_eq!(FONT_NAMES.len(), 289);
    assert_eq!(ORIGINAL_SAMPLES.len(), FONT_NAMES.len());
    let mut fonts = Fonts::new();
    let mut mismatches = Vec::new();
    for ((_, label), (snapshot_label, expected)) in FONT_NAMES.iter().zip(ORIGINAL_SAMPLES) {
        assert_eq!(label, snapshot_label);
        if &fonts.render(label, "23 Quick foxes") != expected {
            mismatches.push(*label);
        }
    }
    assert!(
        mismatches.is_empty(),
        "fonts differing from the original: {mismatches:?}"
    );
}

#[test]
fn timer_fonts_preserve_original_smushing_and_padding() {
    let mut fonts = Fonts::new();
    for (label, expected) in [
        (
            "ANSI Regular",
            include_str!("../assets/fonts/snapshots/ANSI_Regular_time.txt"),
        ),
        (
            "Colossal",
            include_str!("../assets/fonts/snapshots/Colossal_time.txt"),
        ),
        (
            "Slant Relief",
            include_str!("../assets/fonts/snapshots/Slant_Relief_time.txt"),
        ),
        (
            "Standard",
            include_str!("../assets/fonts/snapshots/Standard_time.txt"),
        ),
    ] {
        assert_eq!(fonts.render(label, "55:55"), expected, "{label}");
        // A different countdown must evict only the rendered value, not the font.
        fonts.render(label, "55:54");
        assert_eq!(fonts.render(label, "55:55"), expected, "{label}");
        assert_eq!(fonts.render(label, "55:55"), expected, "{label}");
    }
}

#[test]
fn punctuation_uses_the_original_layout_rules() {
    let mut fonts = Fonts::new();
    for (label, expected) in [
        (
            "ANSI Regular",
            include_str!("../assets/fonts/snapshots/ANSI_Regular_layout.txt"),
        ),
        (
            "Colossal",
            include_str!("../assets/fonts/snapshots/Colossal_layout.txt"),
        ),
        (
            "Slant Relief",
            include_str!("../assets/fonts/snapshots/Slant_Relief_layout.txt"),
        ),
        (
            "Standard",
            include_str!("../assets/fonts/snapshots/Standard_layout.txt"),
        ),
    ] {
        assert_eq!(fonts.render(label, r"AZ_[]/\>< $"), expected, "{label}");
    }
}

#[test]
fn multiline_text_uses_the_original_vertical_layout() {
    let mut fonts = Fonts::new();
    for (label, expected) in [
        (
            "ANSI Regular",
            include_str!("../assets/fonts/snapshots/ANSI_Regular_multiline.txt"),
        ),
        (
            "Colossal",
            include_str!("../assets/fonts/snapshots/Colossal_multiline.txt"),
        ),
        (
            "Slant Relief",
            include_str!("../assets/fonts/snapshots/Slant_Relief_multiline.txt"),
        ),
        (
            "Standard",
            include_str!("../assets/fonts/snapshots/Standard_multiline.txt"),
        ),
    ] {
        assert_eq!(fonts.render(label, "23\nQuick foxes"), expected, "{label}");
    }
}

#[test]
fn extended_glyphs_and_right_to_left_fonts_match_the_original() {
    let mut fonts = Fonts::new();
    for (label, expected) in [
        (
            "Ivrit",
            include_str!("../assets/fonts/snapshots/Ivrit_unicode.txt"),
        ),
        (
            "Banner",
            include_str!("../assets/fonts/snapshots/Banner_unicode.txt"),
        ),
        (
            "Big",
            include_str!("../assets/fonts/snapshots/Big_unicode.txt"),
        ),
        (
            "ANSI Regular",
            include_str!("../assets/fonts/snapshots/ANSI_Regular_unicode.txt"),
        ),
        (
            "Standard",
            include_str!("../assets/fonts/snapshots/Standard_unicode.txt"),
        ),
    ] {
        assert_eq!(fonts.render(label, "ÄÖÜäöüß é אַ"), expected, "{label}");
    }
}
