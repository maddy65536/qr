use std::iter;

use crate::encoding::{self, ECLevel};
use crate::tables::{ALIGNMENT_PATTERNS, BLOCK_GROUPS, VERSION_INFO};
use crate::{bitstream, rsec};

const MASKS: [fn((usize, usize)) -> bool; 8] = [
    |p| (p.0 + p.1) % 2 == 0,
    |p| p.0 % 2 == 0,
    |p| p.1 % 3 == 0,
    |p| (p.0 + p.1) % 3 == 0,
    |p| ((p.0 / 2) + (p.1 / 3)) % 2 == 0,
    |p| (p.0 * p.1) % 2 + (p.0 * p.1) % 3 == 0,
    |p| ((p.0 * p.1) % 2 + (p.0 * p.1) % 3) % 2 == 0,
    |p| ((p.0 + p.1) % 2 + (p.0 * p.1) % 3) % 2 == 0,
];

const FINDER_LIKE_LEN: usize = 11;
const FINDER_LIKE: [[bool; FINDER_LIKE_LEN]; 2] = [
    [
        true, false, true, true, true, false, true, false, false, false, false,
    ],
    [
        false, false, false, false, true, false, true, true, true, false, true,
    ],
];

#[derive(Debug, Clone)]
pub struct Qr {
    pub data: Vec<Vec<bool>>,
    version: usize,
    ec: ECLevel,
}

impl Qr {
    pub fn make_blank(version: usize, ec: ECLevel) -> Self {
        if !(1..=40).contains(&version) {
            panic!("tried to make qr code with invalid version!");
        }
        Self {
            data: make_fixed_patterns(version).unwrap(),
            version,
            ec,
        }
    }

    pub fn make_qr(data: &str, ec: ECLevel) -> Option<Self> {
        // encode data
        let mode = encoding::detect_mode(data);
        println!("mode: {:?}", mode);
        // need a better length calculation for the other modes but it works for now
        let version = encoding::detect_version(mode, data.len(), ec)?;
        println!("version: {:?}", version);
        let encoded = encoding::encode(data.into(), mode, version, ec).unwrap();
        println!("encoded: {:02X?} len: {}", encoded, encoded.len());
        let stream: Vec<bool> = bitstream::Bitstream::from_bytes(&encoded).into();

        // draw qr code
        let mut qr = Self::make_blank(version, ec);
        let order = ModuleOrder::new(version);
        stream
            .iter()
            .zip(order)
            .for_each(|(bit, pos)| qr.data[pos.0][pos.1] = *bit);

        qr = apply_best_mask(&qr);
        Some(qr)
    }

    pub fn score(&self) -> usize {
        score_matrix(&self.data)
    }
}

pub fn version_to_width(version: usize) -> Option<usize> {
    if !(1..=40).contains(&version) {
        None
    } else {
        Some((version * 4) + 17)
    }
}

pub fn get_alignment_locations(version: usize) -> Vec<(usize, usize)> {
    let channels = ALIGNMENT_PATTERNS[version - 1];
    let max = channels.last().unwrap_or(&0); // 0 is just a dummy value here
    let mut res = vec![];
    for row in 0..channels.len() {
        for col in 0..channels.len() {
            if ((channels[col] == *max || channels[col] == 6) && channels[row] == 6)
                || (channels[row] == *max && channels[col] == 6)
            {
                continue;
            }
            res.push((channels[row], channels[col]));
        }
    }

    res
}

pub fn make_fixed_patterns(version: usize) -> Option<Vec<Vec<bool>>> {
    if !(1..=40).contains(&version) {
        return None;
    }
    let max = version_to_width(version)?;

    let mut res: Vec<Vec<bool>> =
        iter::repeat_n(iter::repeat_n(false, max).collect(), max).collect();

    // draw timing patterns
    for (row, module) in res.iter_mut().enumerate() {
        module[6] = row & 1 == 0;
    }
    for col in 0..max {
        res[6][col] = col & 1 == 0;
    }

    // draw finders
    draw_finder(&mut res, (3, 3));
    draw_finder(&mut res, (3, max - 4));
    draw_finder(&mut res, (max - 4, 3));

    // draw alignment patterns
    for pos in get_alignment_locations(version) {
        draw_alignment(&mut res, pos);
    }

    // draw that one module
    res[max - 8][8] = true;

    // draw the version patterns
    draw_version(&mut res, version);

    Some(res)
}

pub fn draw_square(
    data: &mut [Vec<bool>],
    val: bool,
    top_left: (usize, usize),
    bottom_right: (usize, usize),
) {
    for row in data.iter_mut().take(bottom_right.0 + 1).skip(top_left.0) {
        for module in row.iter_mut().take(bottom_right.1 + 1).skip(top_left.1) {
            *module = val;
        }
    }
}

/// will panic if you try to draw it in a place where it would be outside the array
pub fn draw_finder(data: &mut [Vec<bool>], pos: (usize, usize)) {
    draw_square(data, true, (pos.0 - 3, pos.1 - 3), (pos.0 + 3, pos.1 + 3));
    draw_square(data, false, (pos.0 - 2, pos.1 - 2), (pos.0 + 2, pos.1 + 2));
    draw_square(data, true, (pos.0 - 1, pos.1 - 1), (pos.0 + 1, pos.1 + 1));
}

/// will panic if you try to draw it in a place where it would be outside the array
pub fn draw_alignment(data: &mut [Vec<bool>], pos: (usize, usize)) {
    draw_square(data, true, (pos.0 - 2, pos.1 - 2), (pos.0 + 2, pos.1 + 2));
    draw_square(data, false, (pos.0 - 1, pos.1 - 1), (pos.0 + 1, pos.1 + 1));
    data[pos.0][pos.1] = true;
}

fn draw_number(data: &mut [Vec<bool>], num: usize, coords: &[(usize, usize)]) {
    if coords.len() == 18 {
        println!("{:18b}", num);
    }
    for (i, pos) in coords.iter().enumerate() {
        data[pos.0][pos.1] = (num >> (coords.len() - i - 1)) & 1 == 1;
    }
}

fn draw_format(qr: &mut Qr, mask: usize) {
    let form = rsec::qr_format_encode_masked(((qr.ec as usize) << 3) | mask);
    let max = version_to_width(qr.version).unwrap() - 1;

    draw_number(
        &mut qr.data,
        form,
        &[
            (8, 0),
            (8, 1),
            (8, 2),
            (8, 3),
            (8, 4),
            (8, 5),
            (8, 7),
            (8, 8),
            (7, 8),
            (5, 8),
            (4, 8),
            (3, 8),
            (2, 8),
            (1, 8),
            (0, 8),
        ],
    );
    draw_number(
        &mut qr.data,
        form,
        &[
            (max, 8),
            (max - 1, 8),
            (max - 2, 8),
            (max - 3, 8),
            (max - 4, 8),
            (max - 5, 8),
            (max - 6, 8),
            (8, max - 7),
            (8, max - 6),
            (8, max - 5),
            (8, max - 4),
            (8, max - 3),
            (8, max - 2),
            (8, max - 1),
            (8, max),
        ],
    );
}

pub fn draw_version(data: &mut [Vec<bool>], version: usize) {
    if !(7..=40).contains(&version) {
        return;
    }
    let max = version_to_width(version).unwrap();
    let ver = VERSION_INFO[version - 1];

    draw_number(
        data,
        ver,
        &[
            (max - 9, 5),
            (max - 10, 5),
            (max - 11, 5),
            (max - 9, 4),
            (max - 10, 4),
            (max - 11, 4),
            (max - 9, 3),
            (max - 10, 3),
            (max - 11, 3),
            (max - 9, 2),
            (max - 10, 2),
            (max - 11, 2),
            (max - 9, 1),
            (max - 10, 1),
            (max - 11, 1),
            (max - 9, 0),
            (max - 10, 0),
            (max - 11, 0),
        ],
    );

    draw_number(
        data,
        ver,
        &[
            (5, max - 9),
            (5, max - 10),
            (5, max - 11),
            (4, max - 9),
            (4, max - 10),
            (4, max - 11),
            (3, max - 9),
            (3, max - 10),
            (3, max - 11),
            (2, max - 9),
            (2, max - 10),
            (2, max - 11),
            (1, max - 9),
            (1, max - 10),
            (1, max - 11),
            (0, max - 9),
            (0, max - 10),
            (0, max - 11),
        ],
    );
}

pub fn is_alignment_pattern(version: usize, pos: (usize, usize)) -> bool {
    if !(1..=40).contains(&version) {
        panic!("invalid version!")
    }
    let coords = ALIGNMENT_PATTERNS[version - 1];
    let max = coords.last().unwrap_or(&0);
    for i in coords {
        for j in coords {
            // skip fake alignment patterns
            if !(*i != 6 || *j != 6 && j != max) || (i == max && *j == 6) {
                continue;
            }
            if i.abs_diff(pos.0) < 3 && j.abs_diff(pos.1) < 3 {
                return true;
            }
        }
    }

    false
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModuleType {
    Finder,
    Alignment,
    Timing,
    Pixel,
    Version,
    Format,
    Data,
}

/// i thought this would be useful but it hasn't been so far, maybe there will be a use?
pub fn module_type(version: usize, pos: (usize, usize)) -> ModuleType {
    // just kinda give up on invalid ones sorry
    if !(1..=40).contains(&version) {
        panic!("invalid version!")
    }

    let max = version_to_width(version).unwrap();
    if pos.0 >= max || pos.1 >= max {
        panic!("out of bounds! {} ({}, {})", max, pos.0, pos.1)
    }

    // finder patterns
    if ((0..=7).contains(&pos.0) && (0..=7).contains(&pos.1))
        || ((0..=7).contains(&pos.0) && ((max - 8)..=(max - 1)).contains(&pos.1))
        || ((max - 8)..=(max - 1)).contains(&pos.0) && ((0..=7).contains(&pos.1))
    {
        return ModuleType::Finder;
    }

    // alignment patterns
    if is_alignment_pattern(version, pos) {
        return ModuleType::Alignment;
    }

    // timing patterns
    if pos.0 == 6 || pos.1 == 6 {
        return ModuleType::Timing;
    }

    // that one pixel
    if pos.0 == max - 8 && pos.1 == 8 {
        return ModuleType::Pixel;
    }

    // version info for versions > 6
    if version > 6
        && ((((max - 11)..=(max - 9)).contains(&pos.0) && (0..=5).contains(&pos.1))
            || (((0..=5).contains(&pos.0)) && ((max - 11)..=(max - 9)).contains(&pos.1)))
    {
        return ModuleType::Version;
    }

    // format info
    if (pos.0 == 8 && ((0..=8).contains(&pos.1) || ((max - 8)..=(max - 1)).contains(&pos.1)))
        || (pos.1 == 8 && ((0..=8).contains(&pos.0) || ((max - 8)..=(max - 1)).contains(&pos.0)))
    {
        return ModuleType::Format;
    }

    ModuleType::Data
}

/// is this postition a data module for the given version?
pub fn is_data_module(version: usize, pos: (usize, usize)) -> bool {
    module_type(version, pos) == ModuleType::Data
}

pub struct ModuleOrder {
    curr: (usize, usize),
    version: usize,
    done: bool,
}

impl ModuleOrder {
    pub fn new(version: usize) -> Self {
        if !(1..=40).contains(&version) {
            panic!("invalid version!")
        }
        let max = version_to_width(version).unwrap() - 1;
        Self {
            curr: (max, max),
            version,
            done: false,
        }
    }
}

impl Iterator for ModuleOrder {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let max = version_to_width(self.version)? - 1;
        let res = self.curr;
        let mut curr = self.curr;
        loop {
            if curr.0 == max && curr.1 == 0 {
                self.done = true;
                return Some(res);
            }

            // right hand side of a row?
            // opposite if after the vertical timing strip
            if ((curr.1 & 1) == 1) != (curr.1 > 6) {
                // go left
                curr = (curr.0, curr.1 - 1);
            } else {
                // is this an up row or a down row?
                let up = if curr.1 > 6 {
                    ((curr.1 - 1) / 2) & 1 == 1
                } else {
                    (curr.1 / 2) & 1 == 1
                };

                // if at the end of a row move to the next one otherwise keep going
                if (up && curr.0 == 0) || (!up && curr.0 == max) {
                    curr = (curr.0, curr.1 - 1);
                } else if up {
                    curr = (curr.0 - 1, curr.1 + 1);
                } else {
                    curr = (curr.0 + 1, curr.1 + 1)
                }
            }

            // no timing column allowed
            if curr.1 == 6 {
                curr = (curr.0, curr.1 - 1);
            }
            if is_data_module(self.version, curr) {
                self.curr = curr;
                return Some(res);
            }
        }
    }
}

fn apply_best_mask(qr: &Qr) -> Qr {
    let choices = (0..=7).map(|n| apply_mask(qr, n));
    let res = choices.min_by_key(Qr::score).unwrap();
    println!("score: {}", res.score());
    res
}

fn apply_mask(qr: &Qr, mask: usize) -> Qr {
    let mut res = qr.clone();
    draw_format(&mut res, mask);
    for (i, row) in res.data.iter_mut().enumerate() {
        for (j, module) in row.iter_mut().enumerate() {
            if is_data_module(qr.version, (i, j)) {
                *module = *module != MASKS[mask]((i, j));
            }
        }
    }
    res
}

fn score_matrix(data: &[Vec<bool>]) -> usize {
    // calculate horizontal adjacency score
    let mut h_adj = 0;
    for r in data.iter() {
        let mut curr = false;
        let mut count = 0;
        for m in r.iter() {
            if *m != curr {
                curr = *m;
                if count >= 5 {
                    h_adj += count - 2;
                }
                count = 1;
            } else {
                count += 1;
            }
        }
        // take care of end of row
        if count >= 5 {
            h_adj += count - 2;
        }
    }

    // calculate vertical adjacency score
    let mut v_adj = 0;
    for c in 0..data[0].len() {
        let mut curr = false;
        let mut count = 0;
        for r in data.iter() {
            if r[c] != curr {
                curr = r[c];
                if count >= 5 {
                    v_adj += count - 2;
                }
                count = 1;
            } else {
                count += 1;
            }
        }
        // take care of end of column
        if count >= 5 {
            v_adj += count - 2;
        }
    }

    // calculate block score
    let mut block = 0;
    for (i, r) in data[..data.len() - 1].iter().enumerate() {
        for (j, m) in r[..r.len() - 1].iter().enumerate() {
            if (*m && data[i + 1][j] && data[i][j + 1] && data[i + 1][j + 1])
                || (!*m && !data[i + 1][j] && !data[i][j + 1] && !data[i + 1][j + 1])
            {
                block += 3;
            }
        }
    }

    let mut h_finder = 0;
    for r in data.iter() {
        for i in 0..(r.len() - FINDER_LIKE_LEN + 1) {
            let mut found = true;
            for (j, pat) in FINDER_LIKE[0].iter().enumerate() {
                if r[i + j] != *pat {
                    found = false;
                    break;
                }
            }
            if found {
                h_finder += 40;
            }

            found = true;
            for (j, pat) in FINDER_LIKE[1].iter().enumerate() {
                if r[i + j] != *pat {
                    found = false;
                    break;
                }
            }
            if found {
                h_finder += 40;
            }
        }
    }

    let mut v_finder = 0;
    for c in 0..data[0].len() {
        for i in 0..(data.len() - FINDER_LIKE_LEN + 1) {
            let mut found = true;
            for (j, pat) in FINDER_LIKE[0].iter().enumerate() {
                if data[i + j][c] != *pat {
                    found = false;
                    break;
                }
            }
            if found {
                v_finder += 40;
            }

            found = true;
            for (j, pat) in FINDER_LIKE[1].iter().enumerate() {
                if data[i + j][c] != *pat {
                    found = false;
                    break;
                }
            }
            if found {
                v_finder += 40;
            }
        }
    }

    // calculate proportion score
    let num_mod: usize = data.iter().map(|r| r.len()).sum();
    let num_dark: usize = data.iter().map(|r| r.iter().filter(|m| **m).count()).sum();
    let percentage: f32 = (num_dark as f32) / (num_mod as f32);
    let proportion = 10 * ((10.0 - (20.0 * percentage)).abs().round() as usize);

    h_adj + v_adj + block + h_finder + v_finder + proportion
}

#[cfg(test)]
mod tests {
    use crate::qr::get_alignment_locations;

    #[test]
    fn test_alignment_locations_v1() {
        assert_eq!(get_alignment_locations(1), vec![])
    }

    #[test]
    fn test_alignment_locations_v7() {
        assert_eq!(
            get_alignment_locations(7),
            vec![(6, 22), (22, 6), (22, 22), (22, 38), (38, 22), (38, 38)]
        )
    }
}
