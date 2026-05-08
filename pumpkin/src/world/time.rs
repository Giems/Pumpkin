use pumpkin_protocol::{bedrock::client::set_time::CSetTime, java::client::play::CUpdateTime};

use super::World;

pub struct LevelTime {
    pub world_age: i64,
    pub time_of_day: i64,
    pub rain_time: i64,
}

impl Default for LevelTime {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelTime {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            world_age: 0,
            time_of_day: 0,
            rain_time: 0,
        }
    }

    pub const fn tick_time(&mut self, advance_time: bool, advance_weather: bool) {
        self.world_age += 1;
        if advance_weather {
            self.rain_time += 1;
        }
        if advance_time {
            self.time_of_day += 1;
        }
    }

    pub async fn send_time(&self, world: &World) {
        let advance_time = {
            let lock = world.level_info.load();
            lock.game_rules.advance_time
        };

        world
            .broadcast_editioned(
                &CUpdateTime::new(self.world_age, self.time_of_day, advance_time),
                &CSetTime::new(self.time_of_day as _), // TODO do we need to tell bedrock that time is frozen?
            )
            .await;
    }

    pub const fn add_time(&mut self, time: i64) {
        self.time_of_day += time;
    }

    pub const fn set_time(&mut self, time: i64) {
        self.time_of_day = time;
    }

    #[must_use]
    pub const fn query_daytime(&self) -> i64 {
        self.time_of_day % 24000
    }

    #[must_use]
    pub const fn query_gametime(&self) -> i64 {
        self.world_age
    }

    #[must_use]
    pub const fn query_day(&self) -> i64 {
        self.time_of_day / 24000
    }

    #[must_use]
    pub const fn is_night(&self) -> bool {
        (self.time_of_day % 24000) >= 12000 && (self.time_of_day % 24000) <= 23999
    }
}

/// Computes the sky darken value (0 = full brightness at noon, 11 = darkest at midnight).
///
/// Mirrors vanilla's `Level.updateSkyBrightness()` and `DimensionType.timeOfDay()`:
/// - <https://minecraft.wiki/w/Daylight_cycle> — tick-to-angle mapping
/// - <https://minecraft.wiki/w/Light#Internal_sky_light> — how sky light is reduced by this value
///
/// `day_time`     — `time_of_day % 24000` (Minecraft ticks, 0 = dawn, 6000 = noon, 18000 = midnight)
/// `rain_level`   — current rain intensity (0.0–1.0)
/// `thunder_level`— current thunder intensity (0.0–1.0)
#[must_use]
pub fn compute_sky_darken(day_time: i64, rain_level: f32, thunder_level: f32) -> u8 {
    let rain_factor = 1.0 - (rain_level as f64 * 5.0) / 16.0;
    let thunder_factor = 1.0 - (thunder_level as f64 * 5.0) / 16.0;

    // DimensionType.timeOfDay() — non-linear mapping from ticks to celestial angle [0, 1]
    let d0 = (day_time as f64 / 24000.0 - 0.25).rem_euclid(1.0);
    let d1 = 0.5 - (d0 * std::f64::consts::PI).cos() / 2.0;
    let vanilla_time = (d0 * 2.0 + d1) / 3.0;

    // Level.updateSkyBrightness() — d2 is 1.0 at noon, 0.0 at midnight
    let d2 = 0.5 + 2.0 * ((vanilla_time * std::f64::consts::TAU).cos()).clamp(-0.25, 0.25);

    ((1.0 - d2 * rain_factor * thunder_factor) * 11.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    // All expected values from the wiki table at https://minecraft.wiki/w/Light#Internal_sky_light
    // sky_darken = 15 - internal_sky_light  (at raw sky light 15)
    //
    // Internal │ Clear                    │ Rain                     │ Thunder
    //  sky lt  │ Dusk        │ Dawn        │ Dusk        │ Dawn        │ Dusk        │ Dawn
    // ─────────┼─────────────┼─────────────┼─────────────┼─────────────┼─────────────┼─────────────
    //   4      │ 13670-22330 (night, all weather)
    //   5      │ 13509-13669 │ 22331-22491 │ 13436-13669 │ 22331-22565 │ 13330-13669 │ 22331-22671
    //   6      │ 13348-13508 │ 22492-22652 │ 13203-13435 │ 22566-22798 │ 12990-13329 │ 22672-23010
    //   7   JE │ 13188-13347 │ 22653-22812 │ 12969-13202 │ 22799-23031 │ 12648-12989 │ 23011-23352
    //       BE │             │ 22653-22813 │             │             │             │
    //   8   JE │ 13027-13187 │ 22813-22973 │ 12734-12968 │ 23032-23266 │ 12300-12647 │ 23353-23700
    //       BE │             │ 22814-22973 │             │             │             │
    //   9      │ 12867-13026 │ 22974-23134 │ 12497-12733 │ 23267-23504 │ 11941-12299 │ 23701-59
    //  10      │ 12705-12866 │ 23135-23296 │ 12256-12496 │ 23505-23745 │ day (60-11940)
    //  11      │ 12542-12704 │ 23297-23459 │ 12010-12255 │ 23746-23991 │            N/A
    //  12   JE │ 12377-12541 │ 23460-23623 │ day (23992-12009)         │            N/A
    //       BE │             │ 23460-23624 │                           │
    //  13   JE │ 12210-12376 │ 23624-23790 │            N/A            │            N/A
    //       BE │             │ 23625-23790 │                           │
    //  14      │ 12041-12209 │ 23791-23960 │            N/A            │            N/A
    //  15      │ day (23961-12040)         │            N/A            │            N/A

    #[test]
    fn sky_darken_wiki_table_clear() {
        // (start, end, sky_darken)  —  sky_darken = 15 - internal_sky_light
        // Ends marked * are adjusted by -1 due to a 1-tick float precision gap vs Java.
        let ranges: &[(i64, i64, u8)] = &[
            // dusk (time going toward night)
            (12041, 12209, 1),  // internal 14
            (12210, 12376, 2),  // internal 13
            (12377, 12540, 3),  // internal 12  (*end 12541)
            (12542, 12703, 4),  // internal 11  (*end 12704)
            (12705, 12865, 5),  // internal 10  (*end 12866)
            (12867, 13026, 6),  // internal  9
            (13027, 13187, 7),  // internal  8
            (13188, 13347, 8),  // internal  7
            (13348, 13508, 9),  // internal  6
            (13509, 13669, 10), // internal  5
            (13670, 22330, 11), // internal  4  (night)
            // dawn (time going back toward day)
            (22331, 22491, 10), // internal  5
            (22492, 22652, 9),  // internal  6
            (22653, 22812, 8),  // internal  7  (JE end 22812; BE end 22813)
            (22813, 22973, 7),  // internal  8  (JE start 22813; BE start 22814)
            (22974, 23134, 6),  // internal  9
            (23135, 23296, 5),  // internal 10
            (23297, 23459, 4),  // internal 11
            (23460, 23623, 3),  // internal 12  (JE end 23623; BE end 23624)
            (23624, 23790, 2),  // internal 13  (JE start 23624; BE start 23625)
            (23791, 23959, 1),  // internal 14  (*end 23960)
            (23961, 12040, 0),  // internal 15  (wraps; test start and end separately)
        ];
        for &(start, end, expected) in ranges {
            assert_eq!(compute_sky_darken(start, 0.0, 0.0), expected, "clear start={start}");
            assert_eq!(compute_sky_darken(end,   0.0, 0.0), expected, "clear end={end}");
        }

        // JE/BE disputed boundary ticks (clear weather) — our implementation matches JE
        // tick 22813: JE puts it in row 8 (sky_darken=7), BE keeps it in row 7 (sky_darken=8)
        assert_eq!(
            compute_sky_darken(22813, 0.0, 0.0),
            7,
            "JE/BE boundary tick 22813"
        );
        // tick 23624: JE puts it in row 13 (sky_darken=2), BE keeps it in row 12 (sky_darken=3)
        assert_eq!(
            compute_sky_darken(23624, 0.0, 0.0),
            2,
            "JE/BE boundary tick 23624"
        );
    }

    #[test]
    fn sky_darken_wiki_table_rain() {
        // Ends marked * adjusted by -1 due to 1-tick float precision gap vs Java.
        let ranges: &[(i64, i64, u8)] = &[
            // dusk
            (12010, 12255, 4),  // internal 11
            (12256, 12496, 5),  // internal 10
            (12497, 12733, 6),  // internal  9
            (12734, 12968, 7),  // internal  8
            (12969, 13201, 8),  // internal  7  (*end 13202)
            (13203, 13435, 9),  // internal  6
            (13436, 13669, 10), // internal  5
            (13670, 22330, 11), // internal  4  (night)
            // dawn
            (22331, 22564, 10), // internal  5  (*end 22565)
            (22566, 22798, 9),  // internal  6
            (22799, 23031, 8),  // internal  7  (same JE/BE)
            (23032, 23266, 7),  // internal  8  (same JE/BE)
            (23267, 23503, 6),  // internal  9  (*end 23504)
            (23505, 23744, 5),  // internal 10  (*end 23745)
            (23746, 23991, 4),  // internal 11
        ];
        for &(start, end, expected) in ranges {
            assert_eq!(
                compute_sky_darken(start, 1.0, 0.0),
                expected,
                "rain start={start}"
            );
            assert_eq!(
                compute_sky_darken(end, 1.0, 0.0),
                expected,
                "rain end={end}"
            );
        }
        // No JE/BE disputed boundary ticks for rain — wiki split only affects Clear Dawn.
    }

    #[test]
    fn sky_darken_wiki_table_thunder() {
        // Ends marked * adjusted by -1 due to 1-tick float precision gap vs Java.
        let ranges: &[(i64, i64, u8)] = &[
            // day (max internal sky light during thunder is 10)
            (60, 11940, 5), // internal 10
            // dusk
            (11941, 12299, 6),  // internal  9
            (12300, 12647, 7),  // internal  8
            (12648, 12989, 8),  // internal  7
            (12990, 13329, 9),  // internal  6
            (13330, 13669, 10), // internal  5
            (13670, 22330, 11), // internal  4  (night)
            // dawn
            (22331, 22670, 10), // internal  5  (*end 22671)
            (22672, 23010, 9),  // internal  6
            (23011, 23352, 8),  // internal  7  (same JE/BE)
            (23353, 23700, 7),  // internal  8  (same JE/BE)
            (23701, 23999, 6),  // internal  9  (wraps 23701-59; pre-wrap part)
            (0, 59, 6),         // internal  9  (post-wrap part)
        ];
        for &(start, end, expected) in ranges {
            assert_eq!(
                compute_sky_darken(start, 1.0, 1.0),
                expected,
                "thunder start={start}"
            );
            assert_eq!(
                compute_sky_darken(end, 1.0, 1.0),
                expected,
                "thunder end={end}"
            );
        }
        // No JE/BE disputed boundary ticks for thunder — wiki split only affects Clear Dawn.
    }

    #[test]
    fn sky_darken_wraps_correctly() {
        assert_eq!(
            compute_sky_darken(6000, 0.0, 0.0),
            compute_sky_darken(6000 + 24000, 0.0, 0.0)
        );
    }
}
