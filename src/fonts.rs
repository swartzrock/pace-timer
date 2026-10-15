// Font catalog from pace-ts, using FIGlet.js 1.5.2 assets.
use std::collections::HashMap;

/*
    FIGlet.js (a FIGDriver for FIGlet fonts)
    By Patrick Gillespie (patorjk@gmail.com)
    Originally Written For: http://patorjk.com/software/taag/
    License: MIT (with this header staying intact)

    This JavaScript code aims to fully implement the FIGlet spec.
    Full FIGlet spec: http://patorjk.com/software/taag/docs/figfont.txt

    FIGlet fonts are actually kind of complex, which is why you will see
    a lot of code about parsing and interpreting rules. The actual generation
    code is pretty simple and is done near the bottom of the code.
*/
// Default layout rules ported to Rust from FIGlet.js 1.5.2.
// See assets/FIGLET-LICENSE.txt. Original font notices remain in the assets.

#[derive(Default)]
pub struct Fonts {
    // Each parsed font keeps only its latest rendered string, so a countdown
    // does not accumulate a new cache entry every second.
    cache: HashMap<&'static str, CachedFont>,
}

struct CachedFont {
    font: Font,
    latest: Option<(String, String)>,
}

impl Fonts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, font: &str, text: &str) -> String {
        let (_, label, data) = FONT_DATA
            .iter()
            .find(|(_, label, _)| *label == font)
            .expect("unknown bundled FIGlet font");
        let cached = self.cache.entry(label).or_insert_with(|| CachedFont {
            font: Font::parse(data),
            latest: None,
        });
        if let Some((previous, output)) = &cached.latest
            && previous == text
        {
            return output.clone();
        }
        let output = cached.font.render(text);
        cached.latest = Some((text.to_owned(), output.clone()));
        output
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Layout {
    Full,
    Fitting,
    Universal,
    Controlled,
}

type Block = Vec<Vec<char>>;

struct Font {
    hardblank: char,
    height: usize,
    reverse: bool,
    horizontal: Layout,
    vertical: Layout,
    rules: u32,
    glyphs: HashMap<u32, Block>,
}

impl Font {
    fn parse(data: &str) -> Self {
        let normalized = data.replace("\r\n", "\n").replace('\r', "\n");
        let mut lines = normalized.split('\n');
        let header: Vec<_> = lines.next().unwrap().split(' ').collect();
        let height = header[1].parse().unwrap();
        let old_layout: i32 = header[4].parse().unwrap();
        // Preserve FIGlet.js's optional-header handling, including fonts with
        // a print direction but no full-layout field.
        let layout = if header.len() >= 7 {
            header
                .get(7)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0)
        } else {
            old_layout
        };
        let rules = layout.max(0) as u32;
        let horizontal = if rules & 128 != 0 {
            if rules & 63 != 0 {
                Layout::Controlled
            } else {
                Layout::Universal
            }
        } else if rules & 64 != 0 || old_layout == 0 {
            Layout::Fitting
        } else if old_layout == -1 {
            Layout::Full
        } else if rules & 63 != 0 {
            Layout::Controlled
        } else {
            Layout::Universal
        };
        let vertical = if rules & 16384 != 0 {
            if rules & 7936 != 0 {
                Layout::Controlled
            } else {
                Layout::Universal
            }
        } else if rules & 8192 != 0 {
            Layout::Fitting
        } else if rules & 7936 != 0 {
            Layout::Controlled
        } else {
            Layout::Full
        };
        let comments: usize = header[5].parse().unwrap();
        for _ in 0..comments {
            lines.next();
        }
        let mut glyphs = HashMap::new();
        for code in (32..=126).chain([196, 214, 220, 228, 246, 252, 223]) {
            glyphs.insert(code, read_glyph(&mut lines, height));
        }
        while let Some(tag) = lines.next() {
            let tag = tag.split(' ').next().unwrap_or("");
            if tag.starts_with("-0x") || tag.starts_with("-0X") {
                // Negative FIGlet tags contain mapping metadata rather than
                // input characters; consume them and keep later glyphs.
                read_glyph(&mut lines, height);
                continue;
            }
            let code = if let Some(hex) = tag.strip_prefix("0x").or_else(|| tag.strip_prefix("0X"))
            {
                u32::from_str_radix(hex, 16)
            } else if tag.starts_with('0') && tag.len() > 1 {
                u32::from_str_radix(tag, 8)
            } else {
                tag.parse()
            };
            let Ok(code) = code else { break };
            glyphs.insert(code, read_glyph(&mut lines, height));
        }
        Self {
            hardblank: header[0].chars().nth(5).unwrap(),
            height,
            reverse: header.get(6).and_then(|s| s.parse::<u8>().ok()) == Some(1),
            horizontal,
            vertical,
            rules,
            glyphs,
        }
    }

    fn render(&self, text: &str) -> String {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let mut output = Block::new();
        for line in normalized.split('\n') {
            let mut block = vec![Vec::new(); self.height];
            let mut chars: Vec<_> = line.chars().collect();
            if self.reverse {
                chars.reverse();
            }
            for ch in chars {
                let Some(glyph) = self.glyphs.get(&(ch as u32)) else {
                    continue;
                };
                let overlap = block
                    .iter()
                    .zip(glyph)
                    .map(|(left, right)| self.horizontal_overlap(left, right))
                    .min()
                    .unwrap_or(0);
                for (left, right) in block.iter_mut().zip(glyph) {
                    let start = left.len() - overlap;
                    for i in 0..overlap {
                        let a = left[start + i];
                        let b = right.get(i).copied().unwrap_or(' ');
                        left[start + i] =
                            if a != ' ' && b != ' ' && self.horizontal == Layout::Controlled {
                                self.horizontal_smush(a, b)
                                    .unwrap_or_else(|| self.universal(a, b))
                            } else {
                                self.universal(a, b)
                            };
                    }
                    left.extend(right.iter().skip(overlap));
                }
            }
            if block.iter().all(Vec::is_empty) {
                continue;
            }
            for row in &mut block {
                for cell in row {
                    if *cell == self.hardblank {
                        *cell = ' ';
                    }
                }
            }
            if output.is_empty() {
                output = block;
            } else {
                self.append_vertical(&mut output, block);
            }
        }
        output
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn horizontal_overlap(&self, left: &[char], right: &[char]) -> usize {
        if self.horizontal == Layout::Full {
            return 0;
        }
        for distance in 1..=left.len() {
            let mut collision = false;
            for (&a, &b) in left[left.len() - distance..].iter().zip(right) {
                if a == ' ' || b == ' ' {
                    continue;
                }
                collision = true;
                match self.horizontal {
                    Layout::Fitting => return distance - 1,
                    Layout::Universal => {
                        return if a == self.hardblank || b == self.hardblank {
                            distance - 1
                        } else {
                            distance
                        };
                    }
                    Layout::Controlled if self.horizontal_smush(a, b).is_none() => {
                        return distance - 1;
                    }
                    _ => {}
                }
            }
            if collision {
                return distance;
            }
        }
        left.len()
    }

    fn universal(&self, a: char, b: char) -> char {
        if b == ' ' || (b == self.hardblank && a != ' ') {
            a
        } else {
            b
        }
    }

    fn horizontal_smush(&self, a: char, b: char) -> Option<char> {
        if self.rules & 1 != 0 && a == b && a != self.hardblank {
            return Some(a);
        }
        if self.rules & 2 != 0
            && let Some(ch) = underscore(a, b)
        {
            return Some(ch);
        }
        if self.rules & 4 != 0
            && let Some(ch) = hierarchy(a, b)
        {
            return Some(ch);
        }
        if self.rules & 8 != 0 {
            let pairs = "[] {} ()";
            if let (Some(i), Some(j)) = (pairs.find(a), pairs.find(b))
                && i.abs_diff(j) <= 1
            {
                return Some('|');
            }
        }
        if self.rules & 16 != 0 {
            match (a, b) {
                ('/', '\\') => return Some('|'),
                // FIGlet.js 1.5.2 uses indexOf on its pair table, so the
                // repeated slash/backslash entries never select its Y case.
                ('>', '<') => return Some('X'),
                _ => {}
            }
        }
        if self.rules & 32 != 0 && a == self.hardblank && b == self.hardblank {
            return Some(a);
        }
        None
    }

    fn vertical_smush(&self, a: char, b: char) -> Option<char> {
        if self.rules & 4096 != 0 && a == '|' && b == '|' {
            return Some('|');
        }
        if self.rules & 256 != 0 && a == b {
            return Some(a);
        }
        if self.rules & 512 != 0
            && let Some(ch) = underscore(a, b)
        {
            return Some(ch);
        }
        if self.rules & 1024 != 0
            && let Some(ch) = hierarchy(a, b)
        {
            return Some(ch);
        }
        if self.rules & 2048 != 0 && matches!((a, b), ('-', '_') | ('_', '-')) {
            return Some('=');
        }
        None
    }

    // None is an invalid overlap; Some(false) can continue; Some(true) stops.
    fn vertical_collision(&self, top: &[char], bottom: &[char]) -> Option<bool> {
        if self.vertical == Layout::Full || top.is_empty() || bottom.is_empty() {
            return None;
        }
        let mut stop = false;
        for (&a, &b) in top.iter().zip(bottom) {
            if a == ' ' || b == ' ' {
                continue;
            }
            match self.vertical {
                Layout::Fitting => return None,
                Layout::Universal => return Some(true),
                Layout::Controlled => {
                    // FIGlet.js allows vertical bars to slide while testing
                    // overlap, even when the supersmush rule is disabled.
                    if a == '|' && b == '|' {
                        continue;
                    }
                    self.vertical_smush(a, b)?;
                    stop = true;
                }
                Layout::Full => unreachable!(),
            }
        }
        Some(stop)
    }

    fn append_vertical(&self, top: &mut Block, mut bottom: Block) {
        let top_width = top[0].len();
        let bottom_width = bottom[0].len();
        if top_width > bottom_width {
            for row in &mut bottom {
                row.extend(std::iter::repeat_n(' ', top_width - bottom_width));
            }
        } else {
            for row in top.iter_mut() {
                row.extend(std::iter::repeat_n(' ', bottom_width - top_width));
            }
        }
        let mut overlap = top.len();
        'distance: for distance in 1..=top.len() {
            let mut stop = false;
            for (a, b) in top[top.len() - distance..].iter().zip(&bottom) {
                match self.vertical_collision(a, b) {
                    None => {
                        overlap = distance - 1;
                        break 'distance;
                    }
                    Some(end) => stop |= end,
                }
            }
            if stop {
                overlap = distance;
                break;
            }
        }
        let start = top.len() - overlap;
        for (i, row) in bottom.iter().take(overlap).enumerate() {
            let old = &top[start + i];
            top[start + i] = old
                .iter()
                .zip(row)
                .map(|(&a, &b)| {
                    if a == ' ' || b == ' ' || self.vertical != Layout::Controlled {
                        if b == ' ' { a } else { b }
                    } else {
                        self.vertical_smush(a, b).unwrap_or(b)
                    }
                })
                .collect();
        }
        top.extend(bottom.into_iter().skip(overlap));
    }
}

fn read_glyph<'a>(lines: &mut impl Iterator<Item = &'a str>, height: usize) -> Block {
    (0..height)
        .map(|_| {
            let row = lines.next().unwrap_or("");
            let marker = row.chars().last();
            row.trim_end_matches(|ch| Some(ch) == marker)
                .chars()
                .collect()
        })
        .collect()
}

fn underscore(a: char, b: char) -> Option<char> {
    const REPLACEMENTS: &str = "|/\\[]{}()<>";
    if a == '_' && REPLACEMENTS.contains(b) {
        Some(b)
    } else if b == '_' && REPLACEMENTS.contains(a) {
        Some(a)
    } else {
        None
    }
}

fn hierarchy(a: char, b: char) -> Option<char> {
    let classes = "| /\\ [] {} () <>";
    let i = classes.find(a)?;
    let j = classes.find(b)?;
    if i != j && i.abs_diff(j) != 1 {
        Some(if i > j { a } else { b })
    } else {
        None
    }
}

const FONT_DATA: &[(&str, &str, &str)] = &[
    ("ONE_ROW", "1Row", include_str!("../assets/fonts/1Row.flf")),
    ("THREE_D", "3-D", include_str!("../assets/fonts/3-D.flf")),
    (
        "THREED_DIAGONAL",
        "3D Diagonal",
        include_str!("../assets/fonts/3D Diagonal.utf8.flf"),
    ),
    (
        "THREED_ASCII",
        "3D-ASCII",
        include_str!("../assets/fonts/3D-ASCII.flf"),
    ),
    ("THREEX5", "3x5", include_str!("../assets/fonts/3x5.flf")),
    ("FOURMAX", "4Max", include_str!("../assets/fonts/4Max.flf")),
    (
        "FIVE_LINE_OBLIQUE",
        "5 Line Oblique",
        include_str!("../assets/fonts/5 Line Oblique.flf"),
    ),
    (
        "AMC_3_LINE",
        "AMC 3 Line",
        include_str!("../assets/fonts/AMC 3 Line.flf"),
    ),
    (
        "AMC_3_LIV1",
        "AMC 3 Liv1",
        include_str!("../assets/fonts/AMC 3 Liv1.flf"),
    ),
    (
        "AMC_AAA01",
        "AMC AAA01",
        include_str!("../assets/fonts/AMC AAA01.flf"),
    ),
    (
        "AMC_NEKO",
        "AMC Neko",
        include_str!("../assets/fonts/AMC Neko.flf"),
    ),
    (
        "AMC_RAZOR",
        "AMC Razor",
        include_str!("../assets/fonts/AMC Razor.flf"),
    ),
    (
        "AMC_RAZOR2",
        "AMC Razor2",
        include_str!("../assets/fonts/AMC Razor2.flf"),
    ),
    (
        "AMC_SLASH",
        "AMC Slash",
        include_str!("../assets/fonts/AMC Slash.flf"),
    ),
    (
        "AMC_SLIDER",
        "AMC Slider",
        include_str!("../assets/fonts/AMC Slider.flf"),
    ),
    (
        "AMC_THIN",
        "AMC Thin",
        include_str!("../assets/fonts/AMC Thin.flf"),
    ),
    (
        "AMC_TUBES",
        "AMC Tubes",
        include_str!("../assets/fonts/AMC Tubes.flf"),
    ),
    (
        "AMC_UNTITLED",
        "AMC Untitled",
        include_str!("../assets/fonts/AMC Untitled.flf"),
    ),
    (
        "ANSI_REGULAR",
        "ANSI Regular",
        include_str!("../assets/fonts/ANSI Regular.flf"),
    ),
    (
        "ANSI_SHADOW",
        "ANSI Shadow",
        include_str!("../assets/fonts/ANSI Shadow.flf"),
    ),
    (
        "ASCII_NEW_ROMAN",
        "ASCII New Roman",
        include_str!("../assets/fonts/ASCII New Roman.utf8.flf"),
    ),
    (
        "ACROBATIC",
        "Acrobatic",
        include_str!("../assets/fonts/Acrobatic.flf"),
    ),
    (
        "ALLIGATOR",
        "Alligator",
        include_str!("../assets/fonts/Alligator.flf"),
    ),
    (
        "ALLIGATOR2",
        "Alligator2",
        include_str!("../assets/fonts/Alligator2.flf"),
    ),
    ("ALPHA", "Alpha", include_str!("../assets/fonts/Alpha.flf")),
    (
        "ALPHABET",
        "Alphabet",
        include_str!("../assets/fonts/Alphabet.flf"),
    ),
    (
        "ARROWS",
        "Arrows",
        include_str!("../assets/fonts/Arrows.flf"),
    ),
    (
        "AVATAR",
        "Avatar",
        include_str!("../assets/fonts/Avatar.flf"),
    ),
    (
        "B1FF",
        "B1FF",
        include_str!("../assets/fonts/B1FF.utf8.flf"),
    ),
    (
        "BANNER",
        "Banner",
        include_str!("../assets/fonts/Banner.flf"),
    ),
    (
        "BANNER3_D",
        "Banner3-D",
        include_str!("../assets/fonts/Banner3-D.flf"),
    ),
    (
        "BANNER3",
        "Banner3",
        include_str!("../assets/fonts/Banner3.flf"),
    ),
    (
        "BANNER4",
        "Banner4",
        include_str!("../assets/fonts/Banner4.flf"),
    ),
    (
        "BARBWIRE",
        "Barbwire",
        include_str!("../assets/fonts/Barbwire.flf"),
    ),
    ("BASIC", "Basic", include_str!("../assets/fonts/Basic.flf")),
    (
        "BEAR",
        "Bear",
        include_str!("../assets/fonts/Bear.utf8.flf"),
    ),
    ("BELL", "Bell", include_str!("../assets/fonts/Bell.flf")),
    (
        "BENJAMIN",
        "Benjamin",
        include_str!("../assets/fonts/Benjamin.flf"),
    ),
    (
        "BIG_CHIEF",
        "Big Chief",
        include_str!("../assets/fonts/Big Chief.flf"),
    ),
    (
        "BIG_MONEY_NE",
        "Big Money-ne",
        include_str!("../assets/fonts/Big Money-ne.flf"),
    ),
    (
        "BIG_MONEY_NW",
        "Big Money-nw",
        include_str!("../assets/fonts/Big Money-nw.flf"),
    ),
    (
        "BIG_MONEY_SE",
        "Big Money-se",
        include_str!("../assets/fonts/Big Money-se.flf"),
    ),
    (
        "BIG_MONEY_SW",
        "Big Money-sw",
        include_str!("../assets/fonts/Big Money-sw.flf"),
    ),
    ("BIG", "Big", include_str!("../assets/fonts/Big.flf")),
    (
        "BIGFIG",
        "Bigfig",
        include_str!("../assets/fonts/Bigfig.flf"),
    ),
    (
        "BINARY",
        "Binary",
        include_str!("../assets/fonts/Binary.flf"),
    ),
    ("BLOCK", "Block", include_str!("../assets/fonts/Block.flf")),
    (
        "BLOCKS",
        "Blocks",
        include_str!("../assets/fonts/Blocks.utf8.flf"),
    ),
    (
        "BLOODY",
        "Bloody",
        include_str!("../assets/fonts/Bloody.flf"),
    ),
    (
        "BOLGER",
        "Bolger",
        include_str!("../assets/fonts/Bolger.flf"),
    ),
    (
        "BRACED",
        "Braced",
        include_str!("../assets/fonts/Braced.flf"),
    ),
    (
        "BRIGHT",
        "Bright",
        include_str!("../assets/fonts/Bright.flf"),
    ),
    (
        "BROADWAY_KB",
        "Broadway KB",
        include_str!("../assets/fonts/Broadway KB.utf8.flf"),
    ),
    (
        "BROADWAY",
        "Broadway",
        include_str!("../assets/fonts/Broadway.flf"),
    ),
    (
        "BUBBLE",
        "Bubble",
        include_str!("../assets/fonts/Bubble.utf8.flf"),
    ),
    (
        "BULBHEAD",
        "Bulbhead",
        include_str!("../assets/fonts/Bulbhead.flf"),
    ),
    (
        "CALIGRAPHY",
        "Caligraphy",
        include_str!("../assets/fonts/Caligraphy.flf"),
    ),
    (
        "CALIGRAPHY2",
        "Caligraphy2",
        include_str!("../assets/fonts/Caligraphy2.flf"),
    ),
    (
        "CALVIN_S",
        "Calvin S",
        include_str!("../assets/fonts/Calvin S.flf"),
    ),
    ("CARDS", "Cards", include_str!("../assets/fonts/Cards.flf")),
    (
        "CATWALK",
        "Catwalk",
        include_str!("../assets/fonts/Catwalk.flf"),
    ),
    (
        "CHISELED",
        "Chiseled",
        include_str!("../assets/fonts/Chiseled.utf8.flf"),
    ),
    (
        "CHUNKY",
        "Chunky",
        include_str!("../assets/fonts/Chunky.flf"),
    ),
    (
        "COINSTAK",
        "Coinstak",
        include_str!("../assets/fonts/Coinstak.flf"),
    ),
    (
        "COLA",
        "Cola",
        include_str!("../assets/fonts/Cola.utf8.flf"),
    ),
    (
        "COLOSSAL",
        "Colossal",
        include_str!("../assets/fonts/Colossal.flf"),
    ),
    (
        "COMPUTER",
        "Computer",
        include_str!("../assets/fonts/Computer.flf"),
    ),
    (
        "CONTESSA",
        "Contessa",
        include_str!("../assets/fonts/Contessa.flf"),
    ),
    (
        "CONTRAST",
        "Contrast",
        include_str!("../assets/fonts/Contrast.flf"),
    ),
    (
        "COSMIKE",
        "Cosmike",
        include_str!("../assets/fonts/Cosmike.flf"),
    ),
    (
        "CRAWFORD",
        "Crawford",
        include_str!("../assets/fonts/Crawford.flf"),
    ),
    (
        "CRAWFORD2",
        "Crawford2",
        include_str!("../assets/fonts/Crawford2.flf"),
    ),
    (
        "CRAZY",
        "Crazy",
        include_str!("../assets/fonts/Crazy.utf8.flf"),
    ),
    (
        "CRICKET",
        "Cricket",
        include_str!("../assets/fonts/Cricket.flf"),
    ),
    (
        "CURSIVE",
        "Cursive",
        include_str!("../assets/fonts/Cursive.flf"),
    ),
    (
        "CYBERLARGE",
        "Cyberlarge",
        include_str!("../assets/fonts/Cyberlarge.flf"),
    ),
    (
        "CYBERMEDIUM",
        "Cybermedium",
        include_str!("../assets/fonts/Cybermedium.flf"),
    ),
    (
        "CYBERSMALL",
        "Cybersmall",
        include_str!("../assets/fonts/Cybersmall.flf"),
    ),
    (
        "CYGNET",
        "Cygnet",
        include_str!("../assets/fonts/Cygnet.flf"),
    ),
    ("DANC4", "DANC4", include_str!("../assets/fonts/DANC4.flf")),
    (
        "DOS_REBEL",
        "DOS Rebel",
        include_str!("../assets/fonts/DOS Rebel.flf"),
    ),
    (
        "DWHISTLED",
        "DWhistled",
        include_str!("../assets/fonts/DWhistled.utf8.flf"),
    ),
    (
        "DANCING_FONT",
        "Dancing Font",
        include_str!("../assets/fonts/Dancing Font.utf8.flf"),
    ),
    (
        "DECIMAL",
        "Decimal",
        include_str!("../assets/fonts/Decimal.flf"),
    ),
    (
        "DEF_LEPPARD",
        "Def Leppard",
        include_str!("../assets/fonts/Def Leppard.flf"),
    ),
    (
        "DELTA_CORPS_PRIEST_1",
        "Delta Corps Priest 1",
        include_str!("../assets/fonts/Delta Corps Priest 1.flf"),
    ),
    (
        "DIAMOND",
        "Diamond",
        include_str!("../assets/fonts/Diamond.flf"),
    ),
    (
        "DIET_COLA",
        "Diet Cola",
        include_str!("../assets/fonts/Diet Cola.utf8.flf"),
    ),
    (
        "DIGITAL",
        "Digital",
        include_str!("../assets/fonts/Digital.utf8.flf"),
    ),
    ("DOH", "Doh", include_str!("../assets/fonts/Doh.flf")),
    ("DOOM", "Doom", include_str!("../assets/fonts/Doom.flf")),
    (
        "DOT_MATRIX",
        "Dot Matrix",
        include_str!("../assets/fonts/Dot Matrix.flf"),
    ),
    (
        "DOUBLE_SHORTS",
        "Double Shorts",
        include_str!("../assets/fonts/Double Shorts.flf"),
    ),
    (
        "DOUBLE",
        "Double",
        include_str!("../assets/fonts/Double.flf"),
    ),
    (
        "DR_PEPPER",
        "Dr Pepper",
        include_str!("../assets/fonts/Dr Pepper.flf"),
    ),
    (
        "EFTI_CHESS",
        "Efti Chess",
        include_str!("../assets/fonts/Efti Chess.flf"),
    ),
    (
        "EFTI_FONT",
        "Efti Font",
        include_str!("../assets/fonts/Efti Font.flf"),
    ),
    (
        "EFTI_ITALIC",
        "Efti Italic",
        include_str!("../assets/fonts/Efti Italic.flf"),
    ),
    (
        "EFTI_PITI",
        "Efti Piti",
        include_str!("../assets/fonts/Efti Piti.flf"),
    ),
    (
        "EFTI_ROBOT",
        "Efti Robot",
        include_str!("../assets/fonts/Efti Robot.flf"),
    ),
    (
        "EFTI_WALL",
        "Efti Wall",
        include_str!("../assets/fonts/Efti Wall.flf"),
    ),
    (
        "EFTI_WATER",
        "Efti Water",
        include_str!("../assets/fonts/Efti Water.flf"),
    ),
    (
        "ELECTRONIC",
        "Electronic",
        include_str!("../assets/fonts/Electronic.flf"),
    ),
    ("ELITE", "Elite", include_str!("../assets/fonts/Elite.flf")),
    ("EPIC", "Epic", include_str!("../assets/fonts/Epic.flf")),
    (
        "FENDER",
        "Fender",
        include_str!("../assets/fonts/Fender.flf"),
    ),
    (
        "FILTER",
        "Filter",
        include_str!("../assets/fonts/Filter.utf8.flf"),
    ),
    (
        "FIRE_FONT_K",
        "Fire Font-k",
        include_str!("../assets/fonts/Fire Font-k.flf"),
    ),
    (
        "FIRE_FONT_S",
        "Fire Font-s",
        include_str!("../assets/fonts/Fire Font-s.flf"),
    ),
    (
        "FLIPPED",
        "Flipped",
        include_str!("../assets/fonts/Flipped.utf8.flf"),
    ),
    (
        "FLOWER_POWER",
        "Flower Power",
        include_str!("../assets/fonts/Flower Power.utf8.flf"),
    ),
    (
        "FOUR_TOPS",
        "Four Tops",
        include_str!("../assets/fonts/Four Tops.flf"),
    ),
    (
        "FRAKTUR",
        "Fraktur",
        include_str!("../assets/fonts/Fraktur.flf"),
    ),
    (
        "FUN_FACE",
        "Fun Face",
        include_str!("../assets/fonts/Fun Face.utf8.flf"),
    ),
    (
        "FUN_FACES",
        "Fun Faces",
        include_str!("../assets/fonts/Fun Faces.utf8.flf"),
    ),
    ("FUZZY", "Fuzzy", include_str!("../assets/fonts/Fuzzy.flf")),
    (
        "GEORGI16",
        "Georgi16",
        include_str!("../assets/fonts/Georgi16.flf"),
    ),
    (
        "GEORGIA11",
        "Georgia11",
        include_str!("../assets/fonts/Georgia11.flf"),
    ),
    (
        "GHOST",
        "Ghost",
        include_str!("../assets/fonts/Ghost.utf8.flf"),
    ),
    (
        "GHOULISH",
        "Ghoulish",
        include_str!("../assets/fonts/Ghoulish.utf8.flf"),
    ),
    (
        "GLENYN",
        "Glenyn",
        include_str!("../assets/fonts/Glenyn.utf8.flf"),
    ),
    ("GOOFY", "Goofy", include_str!("../assets/fonts/Goofy.flf")),
    (
        "GOTHIC",
        "Gothic",
        include_str!("../assets/fonts/Gothic.flf"),
    ),
    (
        "GRACEFUL",
        "Graceful",
        include_str!("../assets/fonts/Graceful.flf"),
    ),
    (
        "GRADIENT",
        "Gradient",
        include_str!("../assets/fonts/Gradient.flf"),
    ),
    (
        "GRAFFITI",
        "Graffiti",
        include_str!("../assets/fonts/Graffiti.flf"),
    ),
    ("GREEK", "Greek", include_str!("../assets/fonts/Greek.flf")),
    (
        "HEART_LEFT",
        "Heart Left",
        include_str!("../assets/fonts/Heart Left.flf"),
    ),
    (
        "HEART_RIGHT",
        "Heart Right",
        include_str!("../assets/fonts/Heart Right.flf"),
    ),
    (
        "HENRY_3D",
        "Henry 3D",
        include_str!("../assets/fonts/Henry 3D.flf"),
    ),
    ("HEX", "Hex", include_str!("../assets/fonts/Hex.flf")),
    (
        "HIEROGLYPHS",
        "Hieroglyphs",
        include_str!("../assets/fonts/Hieroglyphs.utf8.flf"),
    ),
    (
        "HOLLYWOOD",
        "Hollywood",
        include_str!("../assets/fonts/Hollywood.flf"),
    ),
    (
        "HORIZONTAL_LEFT",
        "Horizontal Left",
        include_str!("../assets/fonts/Horizontal Left.utf8.flf"),
    ),
    (
        "HORIZONTAL_RIGHT",
        "Horizontal Right",
        include_str!("../assets/fonts/Horizontal Right.utf8.flf"),
    ),
    (
        "ICL_1900",
        "ICL-1900",
        include_str!("../assets/fonts/ICL-1900.utf8.flf"),
    ),
    (
        "IMPOSSIBLE",
        "Impossible",
        include_str!("../assets/fonts/Impossible.utf8.flf"),
    ),
    (
        "INVITA",
        "Invita",
        include_str!("../assets/fonts/Invita.flf"),
    ),
    (
        "ISOMETRIC1",
        "Isometric1",
        include_str!("../assets/fonts/Isometric1.flf"),
    ),
    (
        "ISOMETRIC2",
        "Isometric2",
        include_str!("../assets/fonts/Isometric2.flf"),
    ),
    (
        "ISOMETRIC3",
        "Isometric3",
        include_str!("../assets/fonts/Isometric3.flf"),
    ),
    (
        "ISOMETRIC4",
        "Isometric4",
        include_str!("../assets/fonts/Isometric4.flf"),
    ),
    (
        "ITALIC",
        "Italic",
        include_str!("../assets/fonts/Italic.flf"),
    ),
    ("IVRIT", "Ivrit", include_str!("../assets/fonts/Ivrit.flf")),
    (
        "JS_BLOCK_LETTERS",
        "JS Block Letters",
        include_str!("../assets/fonts/JS Block Letters.flf"),
    ),
    (
        "JS_BRACKET_LETTERS",
        "JS Bracket Letters",
        include_str!("../assets/fonts/JS Bracket Letters.flf"),
    ),
    (
        "JS_CAPITAL_CURVES",
        "JS Capital Curves",
        include_str!("../assets/fonts/JS Capital Curves.utf8.flf"),
    ),
    (
        "JS_CURSIVE",
        "JS Cursive",
        include_str!("../assets/fonts/JS Cursive.flf"),
    ),
    (
        "JS_STICK_LETTERS",
        "JS Stick Letters",
        include_str!("../assets/fonts/JS Stick Letters.flf"),
    ),
    ("JACKY", "Jacky", include_str!("../assets/fonts/Jacky.flf")),
    (
        "JAZMINE",
        "Jazmine",
        include_str!("../assets/fonts/Jazmine.flf"),
    ),
    (
        "JERUSALEM",
        "Jerusalem",
        include_str!("../assets/fonts/Jerusalem.flf"),
    ),
    (
        "KATAKANA",
        "Katakana",
        include_str!("../assets/fonts/Katakana.flf"),
    ),
    ("KBAN", "Kban", include_str!("../assets/fonts/Kban.flf")),
    (
        "KEYBOARD",
        "Keyboard",
        include_str!("../assets/fonts/Keyboard.flf"),
    ),
    (
        "KNOB",
        "Knob",
        include_str!("../assets/fonts/Knob.utf8.flf"),
    ),
    (
        "KONTO_SLANT",
        "Konto Slant",
        include_str!("../assets/fonts/Konto Slant.utf8.flf"),
    ),
    (
        "KONTO",
        "Konto",
        include_str!("../assets/fonts/Konto.utf8.flf"),
    ),
    ("LCD", "LCD", include_str!("../assets/fonts/LCD.flf")),
    (
        "LARRY_3D_2",
        "Larry 3D 2",
        include_str!("../assets/fonts/Larry 3D 2.flf"),
    ),
    (
        "LARRY_3D",
        "Larry 3D",
        include_str!("../assets/fonts/Larry 3D.flf"),
    ),
    ("LEAN", "Lean", include_str!("../assets/fonts/Lean.flf")),
    (
        "LETTERS",
        "Letters",
        include_str!("../assets/fonts/Letters.flf"),
    ),
    (
        "LIL_DEVIL",
        "Lil Devil",
        include_str!("../assets/fonts/Lil Devil.utf8.flf"),
    ),
    (
        "LINE_BLOCKS",
        "Line Blocks",
        include_str!("../assets/fonts/Line Blocks.utf8.flf"),
    ),
    ("LINUX", "Linux", include_str!("../assets/fonts/Linux.flf")),
    (
        "LOCKERGNOME",
        "Lockergnome",
        include_str!("../assets/fonts/Lockergnome.flf"),
    ),
    (
        "MADRID",
        "Madrid",
        include_str!("../assets/fonts/Madrid.flf"),
    ),
    (
        "MARQUEE",
        "Marquee",
        include_str!("../assets/fonts/Marquee.flf"),
    ),
    (
        "MAXFOUR",
        "Maxfour",
        include_str!("../assets/fonts/Maxfour.flf"),
    ),
    (
        "MERLIN1",
        "Merlin1",
        include_str!("../assets/fonts/Merlin1.utf8.flf"),
    ),
    (
        "MERLIN2",
        "Merlin2",
        include_str!("../assets/fonts/Merlin2.utf8.flf"),
    ),
    ("MIKE", "Mike", include_str!("../assets/fonts/Mike.flf")),
    ("MINI", "Mini", include_str!("../assets/fonts/Mini.flf")),
    (
        "MIRROR",
        "Mirror",
        include_str!("../assets/fonts/Mirror.flf"),
    ),
    (
        "MNEMONIC",
        "Mnemonic",
        include_str!("../assets/fonts/Mnemonic.flf"),
    ),
    (
        "MODULAR",
        "Modular",
        include_str!("../assets/fonts/Modular.utf8.flf"),
    ),
    ("MORSE", "Morse", include_str!("../assets/fonts/Morse.flf")),
    (
        "MORSE2",
        "Morse2",
        include_str!("../assets/fonts/Morse2.flf"),
    ),
    (
        "MOSCOW",
        "Moscow",
        include_str!("../assets/fonts/Moscow.flf"),
    ),
    (
        "MSHEBREW210",
        "Mshebrew210",
        include_str!("../assets/fonts/Mshebrew210.flf"),
    ),
    (
        "MUZZLE",
        "Muzzle",
        include_str!("../assets/fonts/Muzzle.utf8.flf"),
    ),
    (
        "NSCRIPT",
        "NScript",
        include_str!("../assets/fonts/NScript.flf"),
    ),
    (
        "NT_GREEK",
        "NT Greek",
        include_str!("../assets/fonts/NT Greek.flf"),
    ),
    (
        "NV_SCRIPT",
        "NV Script",
        include_str!("../assets/fonts/NV Script.flf"),
    ),
    (
        "NANCYJ_FANCY",
        "Nancyj-Fancy",
        include_str!("../assets/fonts/Nancyj-Fancy.flf"),
    ),
    (
        "NANCYJ_IMPROVED",
        "Nancyj-Improved",
        include_str!("../assets/fonts/Nancyj-Improved.flf"),
    ),
    (
        "NANCYJ_UNDERLINED",
        "Nancyj-Underlined",
        include_str!("../assets/fonts/Nancyj-Underlined.flf"),
    ),
    (
        "NANCYJ",
        "Nancyj",
        include_str!("../assets/fonts/Nancyj.flf"),
    ),
    (
        "NIPPLES",
        "Nipples",
        include_str!("../assets/fonts/Nipples.flf"),
    ),
    ("O8", "O8", include_str!("../assets/fonts/O8.flf")),
    ("OS2", "OS2", include_str!("../assets/fonts/OS2.flf")),
    ("OCTAL", "Octal", include_str!("../assets/fonts/Octal.flf")),
    ("OGRE", "Ogre", include_str!("../assets/fonts/Ogre.flf")),
    (
        "OLD_BANNER",
        "Old Banner",
        include_str!("../assets/fonts/Old Banner.flf"),
    ),
    ("PAGGA", "Pagga", include_str!("../assets/fonts/Pagga.flf")),
    (
        "PATORJKS_CHEESE",
        "Patorjk's Cheese",
        include_str!("../assets/fonts/Patorjk's Cheese.flf"),
    ),
    (
        "PATORJK_HEX",
        "Patorjk-HeX",
        include_str!("../assets/fonts/Patorjk-HeX.flf"),
    ),
    ("PAWP", "Pawp", include_str!("../assets/fonts/Pawp.flf")),
    (
        "PEAKS_SLANT",
        "Peaks Slant",
        include_str!("../assets/fonts/Peaks Slant.flf"),
    ),
    ("PEAKS", "Peaks", include_str!("../assets/fonts/Peaks.flf")),
    (
        "PEBBLES",
        "Pebbles",
        include_str!("../assets/fonts/Pebbles.flf"),
    ),
    (
        "PEPPER",
        "Pepper",
        include_str!("../assets/fonts/Pepper.flf"),
    ),
    (
        "POISON",
        "Poison",
        include_str!("../assets/fonts/Poison.flf"),
    ),
    ("PUFFY", "Puffy", include_str!("../assets/fonts/Puffy.flf")),
    (
        "PUZZLE",
        "Puzzle",
        include_str!("../assets/fonts/Puzzle.flf"),
    ),
    (
        "PYRAMID",
        "Pyramid",
        include_str!("../assets/fonts/Pyramid.utf8.flf"),
    ),
    (
        "RAMMSTEIN",
        "Rammstein",
        include_str!("../assets/fonts/Rammstein.utf8.flf"),
    ),
    (
        "RECTANGLES",
        "Rectangles",
        include_str!("../assets/fonts/Rectangles.flf"),
    ),
    (
        "RED_PHOENIX",
        "Red Phoenix",
        include_str!("../assets/fonts/Red Phoenix.flf"),
    ),
    (
        "RELIEF",
        "Relief",
        include_str!("../assets/fonts/Relief.flf"),
    ),
    (
        "RELIEF2",
        "Relief2",
        include_str!("../assets/fonts/Relief2.flf"),
    ),
    (
        "REVERSE",
        "Reverse",
        include_str!("../assets/fonts/Reverse.flf"),
    ),
    ("ROMAN", "Roman", include_str!("../assets/fonts/Roman.flf")),
    (
        "ROT13",
        "Rot13",
        include_str!("../assets/fonts/Rot13.utf8.flf"),
    ),
    (
        "ROTATED",
        "Rotated",
        include_str!("../assets/fonts/Rotated.utf8.flf"),
    ),
    (
        "ROUNDED",
        "Rounded",
        include_str!("../assets/fonts/Rounded.flf"),
    ),
    (
        "ROWAN_CAP",
        "Rowan Cap",
        include_str!("../assets/fonts/Rowan Cap.flf"),
    ),
    ("ROZZO", "Rozzo", include_str!("../assets/fonts/Rozzo.flf")),
    ("RUNIC", "Runic", include_str!("../assets/fonts/Runic.flf")),
    ("RUNYC", "Runyc", include_str!("../assets/fonts/Runyc.flf")),
    (
        "S_BLOOD",
        "S Blood",
        include_str!("../assets/fonts/S Blood.flf"),
    ),
    (
        "SL_SCRIPT",
        "SL Script",
        include_str!("../assets/fonts/SL Script.flf"),
    ),
    (
        "SANTA_CLARA",
        "Santa Clara",
        include_str!("../assets/fonts/Santa Clara.flf"),
    ),
    (
        "SCRIPT",
        "Script",
        include_str!("../assets/fonts/Script.flf"),
    ),
    (
        "SERIFCAP",
        "Serifcap",
        include_str!("../assets/fonts/Serifcap.flf"),
    ),
    (
        "SHADOW",
        "Shadow",
        include_str!("../assets/fonts/Shadow.flf"),
    ),
    (
        "SHIMROD",
        "Shimrod",
        include_str!("../assets/fonts/Shimrod.flf"),
    ),
    ("SHORT", "Short", include_str!("../assets/fonts/Short.flf")),
    (
        "SLANT_RELIEF",
        "Slant Relief",
        include_str!("../assets/fonts/Slant Relief.flf"),
    ),
    ("SLANT", "Slant", include_str!("../assets/fonts/Slant.flf")),
    ("SLIDE", "Slide", include_str!("../assets/fonts/Slide.flf")),
    (
        "SMALL_CAPS",
        "Small Caps",
        include_str!("../assets/fonts/Small Caps.utf8.flf"),
    ),
    (
        "SMALL_ISOMETRIC1",
        "Small Isometric1",
        include_str!("../assets/fonts/Small Isometric1.flf"),
    ),
    (
        "SMALL_KEYBOARD",
        "Small Keyboard",
        include_str!("../assets/fonts/Small Keyboard.flf"),
    ),
    (
        "SMALL_POISON",
        "Small Poison",
        include_str!("../assets/fonts/Small Poison.flf"),
    ),
    (
        "SMALL_SCRIPT",
        "Small Script",
        include_str!("../assets/fonts/Small Script.flf"),
    ),
    (
        "SMALL_SHADOW",
        "Small Shadow",
        include_str!("../assets/fonts/Small Shadow.flf"),
    ),
    (
        "SMALL_SLANT",
        "Small Slant",
        include_str!("../assets/fonts/Small Slant.flf"),
    ),
    (
        "SMALL_TENGWAR",
        "Small Tengwar",
        include_str!("../assets/fonts/Small Tengwar.flf"),
    ),
    ("SMALL", "Small", include_str!("../assets/fonts/Small.flf")),
    (
        "SOFT",
        "Soft",
        include_str!("../assets/fonts/Soft.utf8.flf"),
    ),
    ("SPEED", "Speed", include_str!("../assets/fonts/Speed.flf")),
    (
        "SPLIFF",
        "Spliff",
        include_str!("../assets/fonts/Spliff.utf8.flf"),
    ),
    (
        "STACEY",
        "Stacey",
        include_str!("../assets/fonts/Stacey.flf"),
    ),
    (
        "STAMPATE",
        "Stampate",
        include_str!("../assets/fonts/Stampate.flf"),
    ),
    (
        "STAMPATELLO",
        "Stampatello",
        include_str!("../assets/fonts/Stampatello.flf"),
    ),
    (
        "STANDARD",
        "Standard",
        include_str!("../assets/fonts/Standard.flf"),
    ),
    (
        "STAR_STRIPS",
        "Star Strips",
        include_str!("../assets/fonts/Star Strips.utf8.flf"),
    ),
    (
        "STAR_WARS",
        "Star Wars",
        include_str!("../assets/fonts/Star Wars.flf"),
    ),
    (
        "STELLAR",
        "Stellar",
        include_str!("../assets/fonts/Stellar.flf"),
    ),
    (
        "STFOREK",
        "Stforek",
        include_str!("../assets/fonts/Stforek.flf"),
    ),
    (
        "STICK_LETTERS",
        "Stick Letters",
        include_str!("../assets/fonts/Stick Letters.flf"),
    ),
    ("STOP", "Stop", include_str!("../assets/fonts/Stop.flf")),
    (
        "STRAIGHT",
        "Straight",
        include_str!("../assets/fonts/Straight.flf"),
    ),
    (
        "STRONGER_THAN_ALL",
        "Stronger Than All",
        include_str!("../assets/fonts/Stronger Than All.flf"),
    ),
    (
        "SUB_ZERO",
        "Sub-Zero",
        include_str!("../assets/fonts/Sub-Zero.flf"),
    ),
    (
        "SWAMP_LAND",
        "Swamp Land",
        include_str!("../assets/fonts/Swamp Land.utf8.flf"),
    ),
    ("SWAN", "Swan", include_str!("../assets/fonts/Swan.flf")),
    (
        "SWEET",
        "Sweet",
        include_str!("../assets/fonts/Sweet.utf8.flf"),
    ),
    ("THIS", "THIS", include_str!("../assets/fonts/THIS.flf")),
    ("TANJA", "Tanja", include_str!("../assets/fonts/Tanja.flf")),
    (
        "TENGWAR",
        "Tengwar",
        include_str!("../assets/fonts/Tengwar.flf"),
    ),
    (
        "TERM",
        "Term",
        include_str!("../assets/fonts/Term.utf8.flf"),
    ),
    ("TEST1", "Test1", include_str!("../assets/fonts/Test1.flf")),
    (
        "THE_EDGE",
        "The Edge",
        include_str!("../assets/fonts/The Edge.flf"),
    ),
    ("THICK", "Thick", include_str!("../assets/fonts/Thick.flf")),
    ("THIN", "Thin", include_str!("../assets/fonts/Thin.flf")),
    (
        "THORNED",
        "Thorned",
        include_str!("../assets/fonts/Thorned.utf8.flf"),
    ),
    (
        "THREE_POINT",
        "Three Point",
        include_str!("../assets/fonts/Three Point.flf"),
    ),
    (
        "TICKS_SLANT",
        "Ticks Slant",
        include_str!("../assets/fonts/Ticks Slant.flf"),
    ),
    ("TICKS", "Ticks", include_str!("../assets/fonts/Ticks.flf")),
    ("TILES", "Tiles", include_str!("../assets/fonts/Tiles.flf")),
    (
        "TINKER_TOY",
        "Tinker-Toy",
        include_str!("../assets/fonts/Tinker-Toy.flf"),
    ),
    (
        "TOMBSTONE",
        "Tombstone",
        include_str!("../assets/fonts/Tombstone.flf"),
    ),
    (
        "TRAIN",
        "Train",
        include_str!("../assets/fonts/Train.utf8.flf"),
    ),
    ("TREK", "Trek", include_str!("../assets/fonts/Trek.flf")),
    (
        "TSALAGI",
        "Tsalagi",
        include_str!("../assets/fonts/Tsalagi.utf8.flf"),
    ),
    (
        "TUBULAR",
        "Tubular",
        include_str!("../assets/fonts/Tubular.flf"),
    ),
    (
        "TWISTED",
        "Twisted",
        include_str!("../assets/fonts/Twisted.utf8.flf"),
    ),
    (
        "TWO_POINT",
        "Two Point",
        include_str!("../assets/fonts/Two Point.flf"),
    ),
    (
        "USA_FLAG",
        "USA Flag",
        include_str!("../assets/fonts/USA Flag.flf"),
    ),
    (
        "UNIVERS",
        "Univers",
        include_str!("../assets/fonts/Univers.flf"),
    ),
    (
        "VARSITY",
        "Varsity",
        include_str!("../assets/fonts/Varsity.utf8.flf"),
    ),
    ("WAVY", "Wavy", include_str!("../assets/fonts/Wavy.flf")),
    ("WEIRD", "Weird", include_str!("../assets/fonts/Weird.flf")),
    (
        "WET_LETTER",
        "Wet Letter",
        include_str!("../assets/fonts/Wet Letter.utf8.flf"),
    ),
    (
        "WHIMSY",
        "Whimsy",
        include_str!("../assets/fonts/Whimsy.flf"),
    ),
    ("WOW", "Wow", include_str!("../assets/fonts/Wow.utf8.flf")),
];

pub const FONT_NAMES: &[(&str, &str)] = &[
    ("ONE_ROW", "1Row"),
    ("THREE_D", "3-D"),
    ("THREED_DIAGONAL", "3D Diagonal"),
    ("THREED_ASCII", "3D-ASCII"),
    ("THREEX5", "3x5"),
    ("FOURMAX", "4Max"),
    ("FIVE_LINE_OBLIQUE", "5 Line Oblique"),
    ("AMC_3_LINE", "AMC 3 Line"),
    ("AMC_3_LIV1", "AMC 3 Liv1"),
    ("AMC_AAA01", "AMC AAA01"),
    ("AMC_NEKO", "AMC Neko"),
    ("AMC_RAZOR", "AMC Razor"),
    ("AMC_RAZOR2", "AMC Razor2"),
    ("AMC_SLASH", "AMC Slash"),
    ("AMC_SLIDER", "AMC Slider"),
    ("AMC_THIN", "AMC Thin"),
    ("AMC_TUBES", "AMC Tubes"),
    ("AMC_UNTITLED", "AMC Untitled"),
    ("ANSI_REGULAR", "ANSI Regular"),
    ("ANSI_SHADOW", "ANSI Shadow"),
    ("ASCII_NEW_ROMAN", "ASCII New Roman"),
    ("ACROBATIC", "Acrobatic"),
    ("ALLIGATOR", "Alligator"),
    ("ALLIGATOR2", "Alligator2"),
    ("ALPHA", "Alpha"),
    ("ALPHABET", "Alphabet"),
    ("ARROWS", "Arrows"),
    ("AVATAR", "Avatar"),
    ("B1FF", "B1FF"),
    ("BANNER", "Banner"),
    ("BANNER3_D", "Banner3-D"),
    ("BANNER3", "Banner3"),
    ("BANNER4", "Banner4"),
    ("BARBWIRE", "Barbwire"),
    ("BASIC", "Basic"),
    ("BEAR", "Bear"),
    ("BELL", "Bell"),
    ("BENJAMIN", "Benjamin"),
    ("BIG_CHIEF", "Big Chief"),
    ("BIG_MONEY_NE", "Big Money-ne"),
    ("BIG_MONEY_NW", "Big Money-nw"),
    ("BIG_MONEY_SE", "Big Money-se"),
    ("BIG_MONEY_SW", "Big Money-sw"),
    ("BIG", "Big"),
    ("BIGFIG", "Bigfig"),
    ("BINARY", "Binary"),
    ("BLOCK", "Block"),
    ("BLOCKS", "Blocks"),
    ("BLOODY", "Bloody"),
    ("BOLGER", "Bolger"),
    ("BRACED", "Braced"),
    ("BRIGHT", "Bright"),
    ("BROADWAY_KB", "Broadway KB"),
    ("BROADWAY", "Broadway"),
    ("BUBBLE", "Bubble"),
    ("BULBHEAD", "Bulbhead"),
    ("CALIGRAPHY", "Caligraphy"),
    ("CALIGRAPHY2", "Caligraphy2"),
    ("CALVIN_S", "Calvin S"),
    ("CARDS", "Cards"),
    ("CATWALK", "Catwalk"),
    ("CHISELED", "Chiseled"),
    ("CHUNKY", "Chunky"),
    ("COINSTAK", "Coinstak"),
    ("COLA", "Cola"),
    ("COLOSSAL", "Colossal"),
    ("COMPUTER", "Computer"),
    ("CONTESSA", "Contessa"),
    ("CONTRAST", "Contrast"),
    ("COSMIKE", "Cosmike"),
    ("CRAWFORD", "Crawford"),
    ("CRAWFORD2", "Crawford2"),
    ("CRAZY", "Crazy"),
    ("CRICKET", "Cricket"),
    ("CURSIVE", "Cursive"),
    ("CYBERLARGE", "Cyberlarge"),
    ("CYBERMEDIUM", "Cybermedium"),
    ("CYBERSMALL", "Cybersmall"),
    ("CYGNET", "Cygnet"),
    ("DANC4", "DANC4"),
    ("DOS_REBEL", "DOS Rebel"),
    ("DWHISTLED", "DWhistled"),
    ("DANCING_FONT", "Dancing Font"),
    ("DECIMAL", "Decimal"),
    ("DEF_LEPPARD", "Def Leppard"),
    ("DELTA_CORPS_PRIEST_1", "Delta Corps Priest 1"),
    ("DIAMOND", "Diamond"),
    ("DIET_COLA", "Diet Cola"),
    ("DIGITAL", "Digital"),
    ("DOH", "Doh"),
    ("DOOM", "Doom"),
    ("DOT_MATRIX", "Dot Matrix"),
    ("DOUBLE_SHORTS", "Double Shorts"),
    ("DOUBLE", "Double"),
    ("DR_PEPPER", "Dr Pepper"),
    ("EFTI_CHESS", "Efti Chess"),
    ("EFTI_FONT", "Efti Font"),
    ("EFTI_ITALIC", "Efti Italic"),
    ("EFTI_PITI", "Efti Piti"),
    ("EFTI_ROBOT", "Efti Robot"),
    ("EFTI_WALL", "Efti Wall"),
    ("EFTI_WATER", "Efti Water"),
    ("ELECTRONIC", "Electronic"),
    ("ELITE", "Elite"),
    ("EPIC", "Epic"),
    ("FENDER", "Fender"),
    ("FILTER", "Filter"),
    ("FIRE_FONT_K", "Fire Font-k"),
    ("FIRE_FONT_S", "Fire Font-s"),
    ("FLIPPED", "Flipped"),
    ("FLOWER_POWER", "Flower Power"),
    ("FOUR_TOPS", "Four Tops"),
    ("FRAKTUR", "Fraktur"),
    ("FUN_FACE", "Fun Face"),
    ("FUN_FACES", "Fun Faces"),
    ("FUZZY", "Fuzzy"),
    ("GEORGI16", "Georgi16"),
    ("GEORGIA11", "Georgia11"),
    ("GHOST", "Ghost"),
    ("GHOULISH", "Ghoulish"),
    ("GLENYN", "Glenyn"),
    ("GOOFY", "Goofy"),
    ("GOTHIC", "Gothic"),
    ("GRACEFUL", "Graceful"),
    ("GRADIENT", "Gradient"),
    ("GRAFFITI", "Graffiti"),
    ("GREEK", "Greek"),
    ("HEART_LEFT", "Heart Left"),
    ("HEART_RIGHT", "Heart Right"),
    ("HENRY_3D", "Henry 3D"),
    ("HEX", "Hex"),
    ("HIEROGLYPHS", "Hieroglyphs"),
    ("HOLLYWOOD", "Hollywood"),
    ("HORIZONTAL_LEFT", "Horizontal Left"),
    ("HORIZONTAL_RIGHT", "Horizontal Right"),
    ("ICL_1900", "ICL-1900"),
    ("IMPOSSIBLE", "Impossible"),
    ("INVITA", "Invita"),
    ("ISOMETRIC1", "Isometric1"),
    ("ISOMETRIC2", "Isometric2"),
    ("ISOMETRIC3", "Isometric3"),
    ("ISOMETRIC4", "Isometric4"),
    ("ITALIC", "Italic"),
    ("IVRIT", "Ivrit"),
    ("JS_BLOCK_LETTERS", "JS Block Letters"),
    ("JS_BRACKET_LETTERS", "JS Bracket Letters"),
    ("JS_CAPITAL_CURVES", "JS Capital Curves"),
    ("JS_CURSIVE", "JS Cursive"),
    ("JS_STICK_LETTERS", "JS Stick Letters"),
    ("JACKY", "Jacky"),
    ("JAZMINE", "Jazmine"),
    ("JERUSALEM", "Jerusalem"),
    ("KATAKANA", "Katakana"),
    ("KBAN", "Kban"),
    ("KEYBOARD", "Keyboard"),
    ("KNOB", "Knob"),
    ("KONTO_SLANT", "Konto Slant"),
    ("KONTO", "Konto"),
    ("LCD", "LCD"),
    ("LARRY_3D_2", "Larry 3D 2"),
    ("LARRY_3D", "Larry 3D"),
    ("LEAN", "Lean"),
    ("LETTERS", "Letters"),
    ("LIL_DEVIL", "Lil Devil"),
    ("LINE_BLOCKS", "Line Blocks"),
    ("LINUX", "Linux"),
    ("LOCKERGNOME", "Lockergnome"),
    ("MADRID", "Madrid"),
    ("MARQUEE", "Marquee"),
    ("MAXFOUR", "Maxfour"),
    ("MERLIN1", "Merlin1"),
    ("MERLIN2", "Merlin2"),
    ("MIKE", "Mike"),
    ("MINI", "Mini"),
    ("MIRROR", "Mirror"),
    ("MNEMONIC", "Mnemonic"),
    ("MODULAR", "Modular"),
    ("MORSE", "Morse"),
    ("MORSE2", "Morse2"),
    ("MOSCOW", "Moscow"),
    ("MSHEBREW210", "Mshebrew210"),
    ("MUZZLE", "Muzzle"),
    ("NSCRIPT", "NScript"),
    ("NT_GREEK", "NT Greek"),
    ("NV_SCRIPT", "NV Script"),
    ("NANCYJ_FANCY", "Nancyj-Fancy"),
    ("NANCYJ_IMPROVED", "Nancyj-Improved"),
    ("NANCYJ_UNDERLINED", "Nancyj-Underlined"),
    ("NANCYJ", "Nancyj"),
    ("NIPPLES", "Nipples"),
    ("O8", "O8"),
    ("OS2", "OS2"),
    ("OCTAL", "Octal"),
    ("OGRE", "Ogre"),
    ("OLD_BANNER", "Old Banner"),
    ("PAGGA", "Pagga"),
    ("PATORJKS_CHEESE", "Patorjk's Cheese"),
    ("PATORJK_HEX", "Patorjk-HeX"),
    ("PAWP", "Pawp"),
    ("PEAKS_SLANT", "Peaks Slant"),
    ("PEAKS", "Peaks"),
    ("PEBBLES", "Pebbles"),
    ("PEPPER", "Pepper"),
    ("POISON", "Poison"),
    ("PUFFY", "Puffy"),
    ("PUZZLE", "Puzzle"),
    ("PYRAMID", "Pyramid"),
    ("RAMMSTEIN", "Rammstein"),
    ("RECTANGLES", "Rectangles"),
    ("RED_PHOENIX", "Red Phoenix"),
    ("RELIEF", "Relief"),
    ("RELIEF2", "Relief2"),
    ("REVERSE", "Reverse"),
    ("ROMAN", "Roman"),
    ("ROT13", "Rot13"),
    ("ROTATED", "Rotated"),
    ("ROUNDED", "Rounded"),
    ("ROWAN_CAP", "Rowan Cap"),
    ("ROZZO", "Rozzo"),
    ("RUNIC", "Runic"),
    ("RUNYC", "Runyc"),
    ("S_BLOOD", "S Blood"),
    ("SL_SCRIPT", "SL Script"),
    ("SANTA_CLARA", "Santa Clara"),
    ("SCRIPT", "Script"),
    ("SERIFCAP", "Serifcap"),
    ("SHADOW", "Shadow"),
    ("SHIMROD", "Shimrod"),
    ("SHORT", "Short"),
    ("SLANT_RELIEF", "Slant Relief"),
    ("SLANT", "Slant"),
    ("SLIDE", "Slide"),
    ("SMALL_CAPS", "Small Caps"),
    ("SMALL_ISOMETRIC1", "Small Isometric1"),
    ("SMALL_KEYBOARD", "Small Keyboard"),
    ("SMALL_POISON", "Small Poison"),
    ("SMALL_SCRIPT", "Small Script"),
    ("SMALL_SHADOW", "Small Shadow"),
    ("SMALL_SLANT", "Small Slant"),
    ("SMALL_TENGWAR", "Small Tengwar"),
    ("SMALL", "Small"),
    ("SOFT", "Soft"),
    ("SPEED", "Speed"),
    ("SPLIFF", "Spliff"),
    ("STACEY", "Stacey"),
    ("STAMPATE", "Stampate"),
    ("STAMPATELLO", "Stampatello"),
    ("STANDARD", "Standard"),
    ("STAR_STRIPS", "Star Strips"),
    ("STAR_WARS", "Star Wars"),
    ("STELLAR", "Stellar"),
    ("STFOREK", "Stforek"),
    ("STICK_LETTERS", "Stick Letters"),
    ("STOP", "Stop"),
    ("STRAIGHT", "Straight"),
    ("STRONGER_THAN_ALL", "Stronger Than All"),
    ("SUB_ZERO", "Sub-Zero"),
    ("SWAMP_LAND", "Swamp Land"),
    ("SWAN", "Swan"),
    ("SWEET", "Sweet"),
    ("THIS", "THIS"),
    ("TANJA", "Tanja"),
    ("TENGWAR", "Tengwar"),
    ("TERM", "Term"),
    ("TEST1", "Test1"),
    ("THE_EDGE", "The Edge"),
    ("THICK", "Thick"),
    ("THIN", "Thin"),
    ("THORNED", "Thorned"),
    ("THREE_POINT", "Three Point"),
    ("TICKS_SLANT", "Ticks Slant"),
    ("TICKS", "Ticks"),
    ("TILES", "Tiles"),
    ("TINKER_TOY", "Tinker-Toy"),
    ("TOMBSTONE", "Tombstone"),
    ("TRAIN", "Train"),
    ("TREK", "Trek"),
    ("TSALAGI", "Tsalagi"),
    ("TUBULAR", "Tubular"),
    ("TWISTED", "Twisted"),
    ("TWO_POINT", "Two Point"),
    ("USA_FLAG", "USA Flag"),
    ("UNIVERS", "Univers"),
    ("VARSITY", "Varsity"),
    ("WAVY", "Wavy"),
    ("WEIRD", "Weird"),
    ("WET_LETTER", "Wet Letter"),
    ("WHIMSY", "Whimsy"),
    ("WOW", "Wow"),
];

#[cfg(test)]
mod tests {
    use super::Fonts;

    #[test]
    fn countdown_cache_keeps_only_the_latest_text_per_font() {
        let mut fonts = Fonts::new();
        for seconds in (0..60).rev() {
            fonts.render("ANSI Regular", &format!("00:{seconds:02}"));
        }
        assert_eq!(fonts.cache.len(), 1);
        let cached = &fonts.cache["ANSI Regular"];
        assert_eq!(cached.latest.as_ref().unwrap().0, "00:00");
        let parsed_glyphs = cached.font.glyphs[&48].as_ptr();
        let expected = cached.latest.as_ref().unwrap().1.clone();
        assert_eq!(fonts.render("ANSI Regular", "00:00"), expected);
        assert_eq!(
            fonts.cache["ANSI Regular"].font.glyphs[&48].as_ptr(),
            parsed_glyphs
        );
        fonts.render("Colossal", "00:00");
        assert_eq!(fonts.cache.len(), 2);
    }
}
