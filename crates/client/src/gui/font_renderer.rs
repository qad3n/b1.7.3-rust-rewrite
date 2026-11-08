use crate::assets;
use crate::client::ChatAllowedCharacters;
use crate::render::{GLAllocation, RenderEngine, Tessellator};
use crate::settings::GameSettings;
use game::port::text;
use jvm::image::{BufferedImage, ImageIO};
use jvm::io::FileInputStream;
use jvm::string::JString;
use jvm::throwable::{self, OrNpe};
use lwjgl::buffers::IntBuffer;
use lwjgl::opengl::GL11;
use std::rc::Rc;

const SECTION_SIGN: u16 = 0xA7;
const LINE_HEIGHT: i32 = 8;
const CHARACTERS: usize = 256;
const COLOR_CODES: i32 = 32;
const LIST_BUFFER_SIZE: i32 = 1024;
const COLOR_DIGITS: &str = "0123456789abcdef";

thread_local! {
    static ALLOWED: Vec<u16> = ChatAllowedCharacters::allowedCharacters().encode_utf16().collect();
}

fn units(text: &str) -> Vec<u16> { text.encode_utf16().collect() }

fn allowedIndex(character: u16) -> i32
{
    ALLOWED.with(|allowed| allowed.iter().position(|&unit| unit == character).map_or(-1, |index| index as i32))
}

fn hasVisible(text: &[u16]) -> bool { text.iter().any(|&unit| unit > b' ' as u16) }

fn concat(parts: &[&[u16]]) -> Vec<u16> { parts.concat() }

pub struct FontRenderer
{
    m_charWidth: [i32; CHARACTERS],
    pub m_fontTextureName: i32,
    m_fontDisplayLists: i32,
    m_buffer: IntBuffer,
}

impl FontRenderer
{
    pub fn new(options: &GameSettings, fontPath: &str, textureManager: &RenderEngine) -> FontRenderer
    {
        let mut stream: Option<FileInputStream> = assets::getResourceAsStream(fontPath);
        let bufferedimage: BufferedImage = match ImageIO::read(stream.as_mut())
        {
            Ok(image) => image.orNpe(),
            Err(error) => throwable::throw(&throwable::RUNTIME, &error.to_string()),
        };
        let i: i32 = bufferedimage.getWidth();
        let j: i32 = bufferedimage.getHeight();
        let mut aint: Vec<i32> = vec![0; (i * j) as usize];
        bufferedimage.getRGBRegion(0, 0, i, j, &mut aint, 0, i);
        let mut charWidth: [i32; CHARACTERS] = [0; CHARACTERS];
        for (k, width) in charWidth.iter_mut().enumerate()
        {
            let k: i32 = k as i32;
            let l: i32 = k % 16;
            let i1: i32 = k / 16;
            let mut j1: i32 = 7;
            while j1 >= 0
            {
                let k1: i32 = l * 8 + j1;
                let blank: bool = (0..8).all(|l1| aint[(k1 + (i1 * 8 + l1) * i) as usize] & 0xFF == 0);
                if !blank
                {
                    break;
                }
                j1 -= 1;
            }
            if k == 32
            {
                j1 = 2;
            }
            *width = j1 + 2;
        }
        let fontTextureName: i32 = textureManager.allocateAndSetupTexture(Rc::new(bufferedimage));
        let fontDisplayLists: i32 = GLAllocation::generateDisplayLists(CHARACTERS as i32 + COLOR_CODES);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        for l2 in 0..CHARACTERS as i32
        {
            GL11::glNewList(fontDisplayLists + l2, GL11::GL_COMPILE);
            tessellator.startDrawingQuads();
            let j3: i32 = l2 % 16 * 8;
            let l3: i32 = l2 / 16 * 8;
            let f: f32 = 7.99;
            let f1: f32 = 0.0;
            let f2: f32 = 0.0;
            tessellator.addVertexWithUV(
                0.0,
                (0.0 + f) as f64,
                0.0,
                (j3 as f32 / 128.0 + f1) as f64,
                ((l3 as f32 + f) / 128.0 + f2) as f64,
            );
            tessellator.addVertexWithUV(
                (0.0 + f) as f64,
                (0.0 + f) as f64,
                0.0,
                ((j3 as f32 + f) / 128.0 + f1) as f64,
                ((l3 as f32 + f) / 128.0 + f2) as f64,
            );
            tessellator.addVertexWithUV(
                (0.0 + f) as f64,
                0.0,
                0.0,
                ((j3 as f32 + f) / 128.0 + f1) as f64,
                (l3 as f32 / 128.0 + f2) as f64,
            );
            tessellator.addVertexWithUV(
                0.0,
                0.0,
                0.0,
                (j3 as f32 / 128.0 + f1) as f64,
                (l3 as f32 / 128.0 + f2) as f64,
            );
            tessellator.draw();
            GL11::glTranslatef(charWidth[l2 as usize] as f32, 0.0, 0.0);
            GL11::glEndList();
        }
        for i3 in 0..COLOR_CODES
        {
            let k3: i32 = (i3 >> 3 & 1) * 85;
            let mut i4: i32 = (i3 >> 2 & 1) * 170 + k3;
            let mut j4: i32 = (i3 >> 1 & 1) * 170 + k3;
            let mut k4: i32 = (i3 & 1) * 170 + k3;
            if i3 == 6
            {
                i4 += 85;
            }
            let flag1: bool = i3 >= 16;
            if options.m_anaglyph.get()
            {
                let l4: i32 = (i4 * 30 + j4 * 59 + k4 * 11) / 100;
                let i5: i32 = (i4 * 30 + j4 * 70) / 100;
                let k2: i32 = (i4 * 30 + k4 * 70) / 100;
                i4 = l4;
                j4 = i5;
                k4 = k2;
            }
            if flag1
            {
                i4 /= 4;
                j4 /= 4;
                k4 /= 4;
            }
            GL11::glNewList(fontDisplayLists + CHARACTERS as i32 + i3, GL11::GL_COMPILE);
            GL11::glColor3f(i4 as f32 / 255.0, j4 as f32 / 255.0, k4 as f32 / 255.0);
            GL11::glEndList();
        }
        FontRenderer {
            m_charWidth: charWidth,
            m_fontTextureName: fontTextureName,
            m_fontDisplayLists: fontDisplayLists,
            m_buffer: GLAllocation::createDirectIntBuffer(LIST_BUFFER_SIZE),
        }
    }

    pub fn drawStringWithShadow(&self, text: &str, x: i32, y: i32, color: i32)
    {
        self.renderString(text, x + 1, y + 1, color, true);
        self.drawString(text, x, y, color);
    }

    pub fn drawString(&self, text: &str, x: i32, y: i32, color: i32) { self.renderString(text, x, y, color, false); }

    fn flushIfFull(&self)
    {
        if self.m_buffer.remaining() == 0
        {
            self.m_buffer.flip();
            GL11::glCallLists(&self.m_buffer);
            self.m_buffer.clear();
        }
    }

    pub fn renderString(&self, text: &str, x: i32, y: i32, mut color: i32, shadow: bool)
    {
        let text: Vec<u16> = units(text);
        if shadow
        {
            let n: i32 = color & 0xFF000000u32 as i32;
            color = (color & 0xFCFCFC) >> 2;
            color += n;
        }
        GL11::glBindTexture(GL11::GL_TEXTURE_2D, self.m_fontTextureName);
        let n2: f32 = (color >> 16 & 0xFF) as f32 / 255.0;
        let n3: f32 = (color >> 8 & 0xFF) as f32 / 255.0;
        let n4: f32 = (color & 0xFF) as f32 / 255.0;
        let mut n5: f32 = (color >> 24 & 0xFF) as f32 / 255.0;
        if n5 == 0.0
        {
            n5 = 1.0;
        }
        GL11::glColor4f(n2, n3, n4, n5);
        self.m_buffer.clear();
        GL11::glPushMatrix();
        GL11::glTranslatef(x as f32, y as f32, 0.0);
        let digits: Vec<u16> = units(COLOR_DIGITS);
        let mut lowered: Option<JString> = None;
        let mut i: usize = 0;
        while i < text.len()
        {
            while text.len() > i + 1 && text[i] == SECTION_SIGN
            {
                let code: u16 =
                    lowered.get_or_insert_with(|| JString::fromUtf16(text.clone()).toLowerCase()).charAt(i as i32 + 1);
                let index: i32 = digits.iter().position(|&digit| digit == code).map_or(15, |index| index as i32);
                self.m_buffer.put(self.m_fontDisplayLists + CHARACTERS as i32 + index + if shadow { 16 } else { 0 });
                self.flushIfFull();
                i += 2;
            }
            if i < text.len()
            {
                let index2: i32 = allowedIndex(text[i]);
                if index2 >= 0
                {
                    self.m_buffer.put(self.m_fontDisplayLists + index2 + 32);
                }
            }
            self.flushIfFull();
            i += 1;
        }
        self.m_buffer.flip();
        GL11::glCallLists(&self.m_buffer);
        GL11::glPopMatrix();
    }

    pub fn getStringWidth(&self, text: &str) -> i32 { self.widthOf(&units(text)) }

    fn widthOf(&self, text: &[u16]) -> i32
    {
        let mut n: i32 = 0;
        let mut i: usize = 0;
        while i < text.len()
        {
            if text[i] == SECTION_SIGN
            {
                i += 1;
            }
            else
            {
                let index: i32 = allowedIndex(text[i]);
                if index >= 0
                {
                    n += self.m_charWidth[(index + 32) as usize];
                }
            }
            i += 1;
        }
        n
    }

    fn wrapLine(&self, text: &[u16], width: i32, mut row: impl FnMut(&[u16]))
    {
        // port splitUnits gives java String.split on utf16 units
        let words: Vec<Vec<u16>> = text::splitUnits(text, b' ' as u16);
        let space: [u16; 1] = [b' ' as u16];
        let mut j: usize = 0;
        while j < words.len()
        {
            let mut s: Vec<u16> = concat(&[&words[j], &space]);
            j += 1;
            while j < words.len() && self.widthOf(&concat(&[&s, &words[j]])) < width
            {
                s = concat(&[&s, &words[j], &space]);
                j += 1;
            }
            while self.widthOf(&s) > width
            {
                let mut beginIndex: usize = 0;
                while self.widthOf(&s[..beginIndex + 1]) <= width
                {
                    beginIndex += 1;
                }
                if hasVisible(&s[..beginIndex])
                {
                    row(&s[..beginIndex]);
                }
                s = s[beginIndex..].to_vec();
            }
            if hasVisible(&s)
            {
                row(&s);
            }
        }
    }

    pub fn splitAndDraw(&self, text: &str, x: i32, y: i32, width: i32, color: i32)
    {
        self.splitAndDrawUnits(&units(text), x, y, width, color);
    }

    fn splitAndDrawUnits(&self, text: &[u16], x: i32, mut y: i32, width: i32, color: i32)
    {
        // port splitUnits gives java String.split on utf16 units
        let lines: Vec<Vec<u16>> = text::splitUnits(text, b'\n' as u16);
        if lines.len() > 1
        {
            for line in &lines
            {
                self.splitAndDrawUnits(line, x, y, width, color);
                y += self.heightOf(line, width);
            }
            return;
        }
        self.wrapLine(text, width, |row| {
            self.drawString(&String::from_utf16_lossy(row), x, y, color);
            y += LINE_HEIGHT;
        });
    }

    pub fn splitAndGetHeight(&self, text: &str, width: i32) -> i32 { self.heightOf(&units(text), width) }

    fn heightOf(&self, text: &[u16], width: i32) -> i32
    {
        // port splitUnits gives java String.split on utf16 units
        let lines: Vec<Vec<u16>> = text::splitUnits(text, b'\n' as u16);
        if lines.len() > 1
        {
            return lines.iter().map(|line| self.heightOf(line, width)).sum();
        }
        let mut n2: i32 = 0;
        self.wrapLine(text, width, |_| n2 += LINE_HEIGHT);
        if n2 < LINE_HEIGHT
        {
            n2 += LINE_HEIGHT;
        }
        n2
    }
}
