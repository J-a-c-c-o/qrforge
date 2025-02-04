use crate::{QRError, ErrorCorrection, Mode};

pub(crate) fn select_mode(data: &str) -> Mode {
    if data.chars().all(|c| c.is_numeric()) {
        Mode::Numeric
    } else if data.chars().all(|c| ALPHANUMERIC.contains(&c)) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

const ALPHANUMERIC: [char; 45] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
    'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z', ' ', '$', '%', '*',
    '+', '-', '.', '/', ':',
];


pub(crate) fn get_version(data: &str, error_correction: &ErrorCorrection, mode: &Mode) -> Result<usize, QRError> {
    for i in 1..=40 {
        if get_capacity(i, error_correction, mode) >= data.len() {
            return Ok(i);
        }
    }
    Err(QRError::new("Data too long"))
}


pub(crate) fn get_capacity(version: usize, error_correction: &ErrorCorrection, mode: &Mode) -> usize {

    let mode = mode.to_value();

    let error_correction = error_correction.to_value();


    CAPACITY[mode][version - 1][error_correction]
}


const CAPACITY: [[[usize; 4]; 40];4] = [
    //numeric
    [
        //version 1-5
        [41, 34, 27, 17],
        [77, 63, 48, 34],
        [127, 101, 77, 58],
        [187, 149, 111, 82],
        [255, 202, 144, 106],

        //version 6-10
        [322, 255, 178, 139],
        [370, 293, 207, 154],
        [461, 365, 259, 202],
        [552, 432, 312, 235],
        [652, 513, 364, 288],

        //version 11-15
        [772, 604, 427, 331],
        [883, 691, 489, 374],
        [1022, 796, 580, 427],
        [1101, 871, 621, 468],
        [1250, 991, 703, 530],

        //version 16-20
        [1408, 1082, 775, 602],
        [1548, 1212, 876, 674],
        [1725, 1346, 948, 746],
        [1903, 1500, 1063, 813],
        [2061, 1600, 1159, 919],

        //version 21-25
        [2232, 1708, 1224, 969],
        [2409, 1872, 1358, 1056],
        [2620, 2059, 1468, 1108],
        [2812, 2188, 1588, 1228],
        [3057, 2395, 1718, 1286],

        //version 26-30
        [3283, 2544, 1804, 1425],
        [3517, 2701, 1933, 1501],
        [3669, 2857, 2085, 1581],
        [3909, 3035, 2181, 1677],
        [4158, 3289, 2358, 1782],

        //version 31-35
        [4417, 3486, 2473, 1897],
        [4686, 3693, 2670, 2022],
        [4965, 3909, 2805, 2157],
        [5253, 4134, 2949, 2301],
        [5529, 4343, 3081, 2361],

        //version 36-40
        [5836, 4588, 3244, 2524],
        [6153, 4775, 3417, 2625],
        [6479, 5039, 3599, 2735],
        [6743, 5313, 3791, 2927],
        [7089, 5596, 3993, 3057],
    ],
    //alphanumeric
    [
        //version 1-5
        [25, 20, 16, 10],
        [47, 38, 29, 20],
        [77, 61, 47, 35],
        [114, 90, 67, 50],
        [154, 122, 87, 64],

        //version 6-10
        [195, 154, 108, 84],
        [224, 178, 125, 93],
        [279, 221, 157, 122],
        [335, 262, 189, 143],
        [395, 311, 221, 174],

        //version 11-15
        [468, 366, 259, 202],
        [535, 419, 296, 230],
        [619, 483, 352, 273],
        [667, 528, 376, 283],
        [758, 600, 426, 331],

        //version 16-20
        [854, 656, 470, 365],
        [938, 734, 531, 408],
        [1046, 816, 574, 452],
        [1153, 909, 644, 493],
        [1249, 970, 702, 557],

        //version 21-25
        [1352, 1056, 750, 587],
        [1460, 1152, 816, 640],
        [1588, 1228, 909, 672],
        [1704, 1358, 954, 744],
        [1853, 1468, 1063, 779],

        //version 26-30
        [1990, 1548, 1112, 864],
        [2132, 1638, 1168, 910],
        [2223, 1732, 1228, 958],
        [2369, 1839, 1283, 1016],
        [2520, 1994, 1428, 1080],

        //version 31-35
        [2677, 2113, 1499, 1150],
        [2840, 2238, 1618, 1226],
        [3009, 2369, 1700, 1307],
        [3183, 2506, 1787, 1394],
        [3351, 2632, 1867, 1431],

        //version 36-40
        [3537, 2780, 1966, 1530],
        [3729, 2894, 2071, 1591],
        [3927, 3054, 2181, 1658],
        [4087, 3220, 2298, 1774],
        [4296, 3391, 2420, 1852],
    ],

    //byte
    [
        //version 1-5
        [17, 14, 11, 7],
        [32, 26, 20, 14],
        [53, 42, 32, 24],
        [78, 62, 46, 34],
        [106, 84, 60, 44],

        //version 6-10
        [134, 106, 74, 58],
        [154, 122, 86, 64],
        [192, 152, 108, 84],
        [230, 180, 130, 98],
        [271, 213, 151, 119],

        //version 11-15
        [321, 251, 177, 137],
        [367, 287, 203, 155],
        [425, 331, 241, 177],
        [458, 362, 258, 194],
        [520, 412, 292, 220],

        //version 16-20
        [586, 450, 322, 250],
        [644, 504, 364, 280],
        [718, 560, 394, 310],
        [792, 624, 442, 338],
        [858, 666, 482, 382],

        //version 21-25
        [929, 711, 509, 403],
        [1003, 779, 565, 439],
        [1091, 857, 611, 461],
        [1171, 911, 661, 511],
        [1273, 997, 715, 535],

        //version 26-30
        [1367, 1059, 751, 593],
        [1465, 1125, 805, 625],
        [1528, 1190, 868, 658],
        [1628, 1264, 908, 698],
        [1732, 1370, 982, 742],

        //version 31-35
        [1840, 1452, 1030, 790],
        [1952, 1538, 1112, 842],
        [2068, 1628, 1168, 898],
        [2188, 1722, 1228, 958],
        [2303, 1809, 1283, 983],

        //version 36-40
        [2431, 1911, 1351, 1051],
        [2563, 1989, 1423, 1093],
        [2699, 2099, 1499, 1139],
        [2809, 2213, 1579, 1219],
        [2953, 2331, 1663, 1273],

    ],

    //kanji
    [
        //version 1-5
        [10, 8, 7, 4],
        [20, 16, 12, 8],
        [32, 26, 20, 15],
        [48, 38, 28, 21],
        [65, 52, 37, 27],

        //version 6-10
        [82, 65, 45, 36],
        [95, 75, 53, 39],
        [118, 93, 66, 52],
        [141, 111, 80, 60],
        [167, 131, 93, 74],

        //version 11-15
        [198, 155, 109, 85],
        [226, 177, 125, 96],
        [262, 205, 149, 109],
        [282, 223, 159, 120],
        [310, 244, 174, 132],

        //version 16-20
        [338, 262, 180, 139],
        [382, 297, 206, 154],
        [403, 314, 223, 173],
        [439, 346, 250, 191],
        [461, 366, 261, 209],

        //version 21-25
        [511, 397, 283, 221],
        [535, 415, 295, 227],
        [593, 463, 325, 250],
        [625, 490, 349, 280],
        [658, 518, 370, 293],

        //version 26-30
        [698, 542, 385, 308],
        [742, 580, 410, 324],
        [790, 621, 438, 348],
        [842, 661, 474, 369],
        [898, 705, 506, 370],

        //version 31-35
        [958, 751, 535, 408],
        [983, 779, 559, 438],
        [1051, 829, 604, 471],
        [1093, 864, 634, 486],
        [1139, 910, 666, 518],

        //version 36-40
        [1219, 954, 711, 559],
        [1273, 1009, 779, 604],
        [1327, 1041, 746, 586],
        [1373, 1093, 784, 644],
        [1455, 1139, 821, 661],

    ]
];
    
