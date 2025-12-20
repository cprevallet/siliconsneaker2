// Module containing data logic and processing.

use crate::config::Units;
use crate::gui::get_unit_system;
use chrono::{Datelike, NaiveDate, NaiveDateTime, Weekday};
use fitparser::{FitDataRecord, Value, profile::field_types::MesgNum};
use gtk4::DropDown;

//Useful values for plotting a graph. */
pub struct GraphAttributes {
    pub plotvals: Vec<(f32, f32)>,
    pub caption: String,
    pub xlabel: String,
    pub ylabel: String,
    pub plot_range: (std::ops::Range<f32>, std::ops::Range<f32>),
    pub y_formatter: Box<dyn Fn(&f32) -> String>,
    // color: RGBColor,
}

// In memory cache to speed up redraws.
pub struct GraphCache {
    pub distance_pace: GraphAttributes,
    pub distance_heart_rate: GraphAttributes,
    pub distance_cadence: GraphAttributes,
    pub distance_elevation: GraphAttributes,
    pub distance_temperature: GraphAttributes,
    pub time_stamps: Vec<NaiveDateTime>,
}

// In memory cache to speed up redraws.
pub struct MapCache {
    pub run_path: Vec<(f32, f32)>,
}

// Calculate the vector mean and standard deviation.
fn mean_and_standard_deviation(data: &Vec<f32>) -> (Option<f32>, Option<f32>) {
    let count = data.len();
    // Handle empty data case to prevent division by zero.
    if count == 0 {
        return (None, None);
    }
    // Calculate the mean (average).
    let sum: f32 = data.iter().sum();
    let mean = sum / (count as f32);
    // Calculate the variance.
    // Variance is the average of the squared differences from the Mean.
    let squared_differences_sum: f32 = data
        .iter()
        // Map each element to its squared difference from the mean
        .map(|&x| {
            let diff = x - mean;
            diff * diff
        })
        // Sum all the squared differences
        .sum();
    let variance = squared_differences_sum / (count as f32);
    // Standard deviation is the square root of the variance.
    return (Some(mean), Some(variance.sqrt()));
}

// Find the largest non-NaN in vector, or NaN otherwise.
fn max_vec(vector: &Vec<f32>) -> f32 {
    let v = vector.iter().copied().fold(0. / 0., f32::max);
    return v;
}

// Find the largest non-NaN in vector, or NaN otherwise.
fn min_vec(vector: &Vec<f32>) -> f32 {
    let v = vector.iter().copied().fold(0. / 0., f32::min);
    return v;
}

// Find the plot range values.
pub fn set_plot_range(
    data: &Vec<(f32, f32)>,
    zoom_x: f32,
    zoom_y: f32,
) -> (std::ops::Range<f32>, std::ops::Range<f32>) {
    if data.len() == 0 {
        return (0.0..0.0, 0.0..0.0);
    };
    if (zoom_x < 0.01) | (zoom_y < 0.01) {
        panic!("Invalid zoom.")
    }
    // Split vector of tuples into two vecs
    let (x, y): (Vec<_>, Vec<_>) = data.iter().map(|(a, b)| (a, b)).unzip();
    // Find the range of the chart, statistics says 95% should lie between +/3 sigma
    // for a normal distribution.  Let's go with that for the range.
    // Disallow zero, negative values of zoom.
    let xrange: std::ops::Range<f32> = min_vec(&x)..1.0 / zoom_x * max_vec(&x);
    if let (Some(mean_y), Some(sigma_y)) = mean_and_standard_deviation(&y) {
        let yrange: std::ops::Range<f32> =
            mean_y - 2.0 / zoom_y * sigma_y..mean_y + 2.0 / zoom_y * sigma_y;
        return (xrange, yrange);
    } else {
        panic!("Could not determine ranges. Reason unknown.")
    }
}

// Convert speed (m/s) to pace(min/mile, min/km).
pub fn cvt_pace(speed: f32, units: &Units) -> f32 {
    match units {
        Units::US => {
            if speed < 1.00 {
                return 26.8224; //avoid divide by zero
            } else {
                return 26.8224 / speed;
            }
        }
        Units::Metric => {
            if speed < 1.00 {
                return 16.666667; //avoid divide by zero
            } else {
                return 16.666667 / speed;
            }
        }
        Units::None => {
            return speed;
        }
    }
}

// Convert distance meters to miles, km.
pub fn cvt_distance(distance: f32, units: &Units) -> f32 {
    match units {
        Units::US => {
            return distance * 0.00062137119;
        }
        Units::Metric => {
            return distance * 0.001;
        }
        Units::None => {
            return distance;
        }
    }
}

// Convert altitude meters to feet, m.
pub fn cvt_altitude(altitude: f32, units: &Units) -> f32 {
    match units {
        Units::US => {
            return altitude * 3.2808399;
        }
        Units::Metric => {
            return altitude * 1.0;
        }
        Units::None => {
            return altitude;
        }
    }
}

// Convert temperature deg C  to deg F, deg C.
pub fn cvt_temperature(temperature: f32, units: &Units) -> f32 {
    match units {
        Units::US => {
            return temperature * 1.8 + 32.0;
        }
        Units::Metric => {
            return temperature * 1.0;
        }
        Units::None => {
            return temperature;
        }
    }
}

// Convert semi-circles to degrees.
pub fn semi_to_degrees(semi: f32) -> f64 {
    let factor: f64 = 2i64.pow(31u32) as f64;
    let deg_val: f64 = semi as f64 * 180f64 / factor;
    return deg_val;
}

// Convert elapsed time in secs to hr,:min,sec.
pub fn cvt_elapsed_time(time_in_sec: f32) -> (i32, i32, i32) {
    let t = time_in_sec / 3600.0;
    let hr = t.trunc();
    let minsec = t.fract() * 60.0;
    let min = minsec.trunc();
    let s = minsec.fract() * 60.0;
    let sec = s.trunc();
    return (hr as i32, min as i32, sec as i32);
}

// Retrieve converted values to plot from fit file.
pub fn get_xy(
    data: &Vec<FitDataRecord>,
    units_widget: &DropDown,
    x_field_name: &str,
    y_field_name: &str,
) -> Vec<(f32, f32)> {
    let mut x_user: Vec<f32> = Vec::new();
    let mut y_user: Vec<f32> = Vec::new();
    let mut xy_pairs: Vec<(f32, f32)> = Vec::new();
    // Get the enumerated value for the unit system the user selected.
    let user_unit = get_unit_system(units_widget);
    // Parameter can be distance, heart_rate, enhanced_speed, enhanced_altitude.
    let x: Vec<f64> = get_msg_record_field_as_vec(&data, x_field_name);
    let y: Vec<f64> = get_msg_record_field_as_vec(&data, y_field_name);
    //  Convert values to 32 bit and create a tuple.
    // Occasionally we see off by one errors in the data.
    // If true, Chop the last one. Must be careful comparing usize values.
    let mut data_range = 0..0;
    if x.len() != 0 {
        data_range = 0..x.len() - 1;
    }
    if x.len() > y.len() && (x.len() != 0) && (y.len() != 0) {
        data_range = 0..y.len() - 1;
    }
    // Doing this in a single pass is less expensive.
    if (x.len() != 0) && (y.len() != 0) {
        for index in data_range.clone() {
            match x_field_name {
                "distance" => {
                    x_user.push(cvt_distance(x[index] as f32, &user_unit));
                }
                _ => {
                    x_user.push(x[index] as f32);
                }
            }
            match y_field_name {
                "enhanced_speed" => {
                    y_user.push(cvt_pace(y[index] as f32, &user_unit));
                }
                "enhanced_altitude" => {
                    y_user.push(cvt_altitude(y[index] as f32, &user_unit));
                }
                "temperature" => {
                    y_user.push(cvt_temperature(y[index] as f32, &user_unit));
                }
                _ => {
                    y_user.push(y[index] as f32);
                }
            }
        }
    }
    if (x_user.len() != 0) && (y_user.len() != 0) {
        for index in data_range.clone() {
            xy_pairs.push((x_user[index], y_user[index]));
        }
    }
    return xy_pairs;
}

// Return a session values of "field_name".
pub fn get_sess_record_field(data: &Vec<FitDataRecord>, field_name: &str) -> f64 {
    for item in data {
        match item.kind() {
            // Individual msgnum::records
            MesgNum::Session => {
                // Retrieve the FitDataField struct.
                for fld in item.fields().iter() {
                    if fld.name() == field_name {
                        return fld.value().clone().try_into().unwrap();
                    }
                }
            }
            _ => (), // matches other patterns
        }
    }
    return f64::NAN;
}

// Return a vector of values of "field_name".
fn get_msg_record_field_as_vec(data: &Vec<FitDataRecord>, field_name: &str) -> Vec<f64> {
    let mut field_vals: Vec<f64> = Vec::new();
    for item in data {
        match item.kind() {
            // Individual msgnum::records
            MesgNum::Record => {
                // Retrieve the FitDataField struct.
                for fld in item.fields().iter() {
                    if fld.name() == field_name {
                        let v64: f64 = fld.value().clone().try_into().unwrap();
                        field_vals.push(v64);
                    }
                }
            }
            _ => (), // matches other patterns
        }
    }
    return field_vals;
}

// Convert various numeric Value variants to f64.
fn extract_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Float64(v) => Some(*v),
        Value::Float32(v) => Some(*v as f64),
        Value::UInt8(v) => Some(*v as f64),
        Value::UInt16(v) => Some(*v as f64),
        Value::UInt32(v) => Some(*v as f64),
        Value::UInt64(v) => Some(*v as f64),
        Value::SInt8(v) => Some(*v as f64),
        Value::SInt16(v) => Some(*v as f64),
        Value::SInt32(v) => Some(*v as f64),
        Value::SInt64(v) => Some(*v as f64),
        _ => None,
    }
}
// Extract a Value Array to a vector of f64.
fn extract_vector_f64(value: &Value) -> Vec<f64> {
    match value {
        Value::Array(arr) => arr.iter().filter_map(extract_f64).collect(),
        _ => Vec::new(),
    }
}

// Return a time for heart rate in the time_in_zone record.
pub fn get_time_in_zone_field(data: &Vec<FitDataRecord>) -> (Option<Vec<f64>>, Option<Vec<f64>>) {
    let mut result: (Option<Vec<f64>>, Option<Vec<f64>>) = (None, None);
    for item in data {
        match item.kind() {
            // Individual msgnum::records
            MesgNum::TimeInZone => {
                // Retrieve the FitDataField struct.
                for fld in item.fields().iter() {
                    if fld.name() == "reference_mesg" && fld.value().to_string() == "session" {
                        let floats: Vec<f64> = extract_vector_f64(item.fields()[2].value());
                        let hr_limits: Vec<f64> = extract_vector_f64(item.fields()[3].value());
                        result = (Some(floats), Some(hr_limits));
                    }
                }
            }
            _ => (), // matches other patterns
        }
    }
    return result;
}

// Return the date a run started on.
pub fn get_run_start_date(data: &Vec<FitDataRecord>) -> (i32, u32, u32) {
    let mut month = 0;
    let mut day = 0;
    let mut year = 0;
    for item in data {
        match item.kind() {
            MesgNum::Session => {
                for fld in item.fields().iter() {
                    if fld.name() == "start_time" {
                        let time_stamp = fld.value().clone().to_string();
                        let from: Result<NaiveDateTime, chrono::ParseError> =
                            NaiveDateTime::parse_from_str(&time_stamp, "%Y-%m-%d %H:%M:%S %z");
                        match from {
                            Ok(date_time) => {
                                year = date_time.date().year();
                                month = date_time.date().month();
                                day = date_time.date().day();
                            }
                            Err(_e) => {
                                panic!("Couldn't parse time_stamp.");
                            }
                        };
                    }
                }
            }
            _ => {}
        }
    }
    return (year, month, day);
}

// Return the date a run started on.
pub fn get_timestamps(data: &Vec<FitDataRecord>) -> Vec<NaiveDateTime> {
    let mut timestamps: Vec<NaiveDateTime> = vec![];
    for item in data {
        match item.kind() {
            MesgNum::Record => {
                for fld in item.fields().iter() {
                    if fld.name() == "timestamp" {
                        let time_stamp = fld.value().clone().to_string();
                        let from: Result<NaiveDateTime, chrono::ParseError> =
                            NaiveDateTime::parse_from_str(&time_stamp, "%Y-%m-%d %H:%M:%S %z");
                        match from {
                            Ok(date_time) => {
                                timestamps.push(date_time);
                            }
                            Err(_e) => {
                                panic!("Couldn't parse time_stamp.");
                            }
                        };
                    }
                }
            }
            _ => {}
        }
    }
    return timestamps;
}
// Determines if a given year, month, and day corresponds to American Thanksgiving.
pub fn is_american_thanksgiving(year: i32, month: u32, day: u32) -> bool {
    // Thanksgiving is always in November
    if month != 11 {
        return false;
    }
    // Basic validation for the day range (Nov 22 - Nov 28)
    if day < 22 || day > 28 {
        return false;
    }
    // Get the date object for November 1st of that year
    if let Some(first_of_nov) = NaiveDate::from_ymd_opt(year, 11, 1) {
        let first_weekday = first_of_nov.weekday();
        // Calculate days from the 1st to the first Thursday
        // Weekday::Thu is represented as 3 (0-indexed starting Mon)
        let days_until_first_thursday =
            (Weekday::Thu.num_days_from_monday() + 7 - first_weekday.num_days_from_monday()) % 7;
        // The first Thursday is (1 + offset)
        // The fourth Thursday is (1 + offset + 21)
        let thanksgiving_day = 1 + days_until_first_thursday + 21;
        return day == thanksgiving_day;
    }
    false
}

// Calculate if a given year, month, and  day falls on easter.
pub fn is_easter(year: i32, month: u32, day: u32) -> bool {
    //Algorithm from https://en.wikipedia.org/wiki/Date_of_Easter#Algorithms
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    // type conversion to unsigned int
    let easter_mon: u32 = ((h + l - 7 * m + 114) / 31).try_into().unwrap();
    let easter_day: u32 = (((h + l - 7 * m + 114) % 31) + 1).try_into().unwrap();
    if day == easter_day && month == easter_mon {
        return true;
    } else {
        return false;
    }
}
