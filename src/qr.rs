use std::{iter, path::Iter};

use crate::tables::ALIGNMENT_PATTERNS;

#[derive(Debug)]
pub struct Qr {
    pub data: Vec<Vec<bool>>,
    version: usize,
}

impl Qr {
    pub fn make_blank(version: usize) -> Self {
        if !(1..=40).contains(&version) {
            panic!("tried to make qr code with invalid version!");
        }
        Self {
            data: make_fixed_patterns(version).unwrap(),
            version,
        }
    }

    pub fn make_test_data() -> Self {
        Self {
            data: vec![
                vec![false, true, false, true, false],
                vec![false, true, false, true, false],
                vec![false, false, false, false, false],
                vec![true, false, false, false, true],
                vec![false, true, true, true, false],
            ],
            version: 0,
        }
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

/// is this postition a data module for the given version?
pub fn is_data_module(version: usize, pos: (usize, usize)) -> bool {
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
        return false;
    }

    // alignment patterns
    if is_alignment_pattern(version, pos) {
        return false;
    }

    // timing patterns
    if pos.0 == 6 || pos.1 == 6 {
        return false;
    }

    // that one pixel
    if pos.0 == max - 8 && pos.1 == 8 {
        return false;
    }

    // version info for versions > 6
    if version > 6
        && ((((max - 11)..=(max - 9)).contains(&pos.0) && (0..=5).contains(&pos.1))
            || (((0..=5).contains(&pos.0)) && ((max - 11)..=(max - 9)).contains(&pos.1)))
    {
        return false;
    }

    // format info
    if (pos.0 == 8 && ((0..=8).contains(&pos.1) || ((max - 8)..=(max - 1)).contains(&pos.1)))
        || (pos.1 == 8 && ((0..=8).contains(&pos.0) || ((max - 8)..=(max - 1)).contains(&pos.0)))
    {
        return false;
    }

    true
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
