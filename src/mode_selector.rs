use crate::{ErrorCorrection, Mode, QRError};

pub(crate) fn select_mode(data: &[u8]) -> Mode {
    if data.iter().all(|&c| c >= b'0' && c <= b'9') {
        Mode::Numeric
    } else if data.iter().all(|&c| ALPHANUMERIC.contains(&(c as char))) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

const ALPHANUMERIC: [char; 45] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ', '$',
    '%', '*', '+', '-', '.', '/', ':',
];

pub(crate) fn get_version(
    segments: &[(Mode, &[u8])],
    error_correction: &ErrorCorrection,
) -> Result<usize, QRError> {
    Ok(5)
}

const CAPACITY: [[usize; 4]; 40] = [
    [152, 128, 104, 72], //version 1
    [272, 224, 176, 128], //version 2
    [440, 352, 272, 208], //version 3
    [640, 512, 384, 288], //version 4
    [864, 688, 496, 368], //version 5
    [1088, 864, 608, 480], //version 6
    [1248, 992, 704, 528], //version 7
    [1552, 1232, 880, 688], //version 8
    [1856, 1456, 1056, 800], //version 9
    [2192, 1728, 1232, 976], //version 10
    [2592, 2032, 1440, 1120], //version 11
    [2960, 2320, 1648, 1264], //version 12
    [3424, 2672, 1952, 1440], //version 13
    [3688, 2920, 2088, 1576], //version 14
    [4184, 3320, 2360, 1784], //version 15
    [4712, 3624, 2600, 2024], //version 16
    [5176, 4056, 2936, 2264], //version 17
    [5768, 4504, 3176, 2504], //version 18
    [6360, 5016, 3560, 2728], //version 19
    [6888, 5352, 3880, 3080], //version 20
    [7456, 5712, 4096, 3248], //version 21
    [8048, 6256, 4544, 3536], //version 22
    [8752, 6880, 4912, 3712], //version 23
    [9392, 7312, 5312, 4112], //version 24
    [10208, 8000, 5744, 4304], //version 25
    [10960, 8496, 6032, 4768], //version 26
    [11744, 9024, 6464, 5024], //version 27
    [12248, 9544, 6968, 5288], //version 28
    [13048, 10136, 7288, 5608], //version 29
    [13880, 10984, 7880, 5960], //version 30
    [14744, 11640, 8264, 6344], //version 31
    [15640, 12328, 8920, 6760], //version 32
    [16568, 13048, 9368, 7208], //version 33
    [17528, 13800, 9848, 7688], //version 34
    [18448, 14496, 10288, 7888], //version 35
    [19472, 15312, 10832, 8432], //version 36
    [20528, 15936, 11408, 8768], //version 37
    [21616, 16816, 12016, 9136], //version 38
    [22496, 17728, 12656, 9776], //version 39
    [23648, 18672, 13328, 10208], //version 40
];
