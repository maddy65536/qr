use std::iter;

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
        Some(((version * 4) + 17) as usize)
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
