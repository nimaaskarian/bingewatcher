// vim:foldmethod=marker
// imports{{{
mod season;
use core::fmt;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use clap::ValueEnum;
pub use season::Season;
//}}}

#[derive(Debug, Clone, ValueEnum)]
pub enum PrintMode {
    Normal,
    #[value(alias="x")]
    Extended,
    #[value(alias="e")]
    NextEpisode,
    #[value(alias="s")]
    Season,
    #[value(alias="E")]
    Episode,
    #[value(alias="p")]
    Path,
    #[value(alias="n")]
    Name,
    #[value(alias="c")]
    Content,
}

#[derive(Debug, Default, PartialEq)]
pub struct Serie {
    seasons: Vec<Season>,
    pub name: String,
    current_season: Option<usize>,
}

#[inline(always)]
fn number_width(mut number: usize) -> usize {
    let mut count = 0;

    while number != 0 {
        number /= 10;
        count += 1;
    }
    match count {
        0 | 1 => 2,
        any => any,
    }
}

impl fmt::Display for Serie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for season in &self.seasons {
            writeln!(f, "{}", season)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SerieParseError {
    EmptyFile,
    ParseFailed,
}

impl FromStr for Serie {
    type Err = SerieParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut seasons = Vec::with_capacity(value.lines().count());
        for line in value.lines() {
            match line.parse() {
                Ok(season) => seasons.push(season),
                Err(season::SeasonError::MalformedSeason) => return Err(SerieParseError::ParseFailed),
                // ignore empty lines
                Err(season::SeasonError::EmptySeason) => {}
            }
        }
        if seasons.is_empty() {
            return Err(SerieParseError::EmptyFile);
        }
        Ok(Serie::new(seasons, ""))
    }
}

impl Serie {
    pub fn new<S: AsRef<str>>(seasons: Vec<Season>, string_like: S) -> Self {
        let name = string_like.as_ref().to_string();
        Serie {
            current_season: seasons.iter().position(Season::is_not_finished),
            seasons,
            name,
        }
    }

    #[inline]
    pub fn print(&self, print: &PrintMode, path: Option<&PathBuf>) {
        match print {
            PrintMode::Extended => self.print_extended(),
            PrintMode::NextEpisode => println!("{}", self.next_episode_str().expect("Serie is finished")),
            PrintMode::Normal => println!("{} {}", self.name, self.next_episode_flat()),
            PrintMode::Season => println!("{}", self.next_season()),
            PrintMode::Episode => println!("{}", self.next_episode()),
            PrintMode::Path => println!("{}", path.unwrap().to_str().unwrap()),
            PrintMode::Name => println!("{}", self.name),
            PrintMode::Content => println!("{self}"),
        }
    }

    #[inline]
    fn next_episode_flat(&self) -> String {
        self.next_episode_str().unwrap_or("FINISHED".to_string())
    }

    #[inline]
    pub fn write(&self, path: PathBuf) -> io::Result<()> {
        let mut file = File::create(path)?;
        write!(file, "{}", self)?;
        Ok(())
    }

    #[inline]
    pub fn from_file(path: &Path) -> Option<Self> {
        let file_content = fs::read_to_string(path).ok()?;

        let serie = file_content.parse::<Self>().ok()?;
        Some(Self {
            name: path.file_stem().unwrap().to_str().unwrap().to_string(),
            ..serie
        })
    }

    #[inline]
    pub fn matches(&self, search: &str) -> bool {
        self.name.to_lowercase().contains(&search.to_lowercase())
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        if let Some(season) = self.seasons.last() {
            season.episodes == season.watched
        } else {
            true
        }
    }

    #[inline]
    pub fn is_not_finished(&self) -> bool {
        !self.is_finished()
    }

    #[inline]
    pub fn next_season(&self) -> usize {
        self.current_season.map(|i| i+1).unwrap_or(0)
    }

    #[inline]
    pub fn next_episode(&self) -> usize {
        match self.current_season() {
            Some(season) => season.watched + 1,
            None => 1,
        }
    }

    #[inline]
    pub fn next_episode_str(&self) -> Option<String> {
        let season = self.current_season()?;
        let episode_width = number_width(season.episodes);
        let season_width = number_width(self.seasons.len());
        Some(format!(
            "S{:0season_width$}E{:0episode_width$}",
            self.next_season(),
            season.watched + 1
        ))
    }

    #[inline]
    pub fn current_season(&self) -> Option<&Season> {
        let index = self.current_season?;
        Some(&self.seasons[index])
    }

    #[inline]
    pub fn print_extended(&self) {
        println!(
            "Name: {}
Percentage: {:.2}%
Watched/Total: {}/{}
Next episode: {}
",
            self.name,
            self.watched_percentage(),
            self.total_watched(),
            self.total_episodes(),
            self.next_episode_flat(),
        );
        for (season, i) in self.seasons.iter().zip(1..) {
            println!("{}: {}",i, season);
        }
    }

    #[inline]
    pub fn unwatch(&mut self, count: usize) -> usize {
        let mut unwatch_count = count;
        let mut index = self.current_season.unwrap_or(self.seasons.len()-1);

        loop {
            unwatch_count = self.seasons[index].unwatch(unwatch_count);
            if unwatch_count == 0 || index == 0 {
                break;
            }
            index -= 1;
        }
        self.current_season = Some(index);
        unwatch_count
    }

    #[inline]
    pub fn watch(&mut self, count: usize) -> usize {
        let mut watch_count = count;
        if let Some(mut index) = self.current_season {
            while index < self.seasons.len() && watch_count > 0 {
                watch_count = self.seasons[index].watch(watch_count);
                index += 1;
            }
            self.current_season = Some(index);
            if !self.seasons[index-1].is_finished() {
                self.current_season = Some(index-1);
            } else if index >= self.seasons.len() {
                self.current_season = None;
            }
            
        }
        watch_count
    }

    #[inline]
    pub fn filename(&self) -> String {
        format!("{}.bw", self.name)
    }

    #[inline]
    pub fn watched_percentage(&self) -> f32 {
        self.total_watched() as f32 / self.total_episodes() as f32 * 100.
    }

    #[inline]
    pub fn total_watched(&self) -> usize {
        self.seasons.iter().map(|season| season.watched).sum()
    }

    #[inline]
    pub fn total_episodes(&self) -> usize {
        self.seasons.iter().map(|season| season.episodes).sum()
    }

    #[inline]
    pub fn merge_serie(&mut self, other: &Serie) {
        let last_index = self.seasons.len()-1;
        let mut iter = other.seasons.iter().skip(last_index);
        if let Some(season) = iter.next() {
            if season.episodes > self.seasons[last_index].episodes {
                self.seasons[last_index].episodes = season.episodes;
            }
        }
        self.seasons.extend(iter.cloned());
    }
}

#[cfg(test)]
mod tests {
    use io::BufWriter;

    use super::*;

    fn get_test_serie() -> Serie {
        "10/20
0/20"
            .parse()
            .unwrap()
    }

    #[test]
    fn test_name() {
        let test = Serie {
            name: "Breaking Bad".to_string(),
            ..Default::default() 
        };
        assert_eq!(test.name, "Breaking Bad")
    }

    #[test]
    fn test_filename() {
        let test = Serie {
            name: "Breaking Bad".to_string(),
            ..Default::default() 
        };
        assert_eq!(test.filename(), "Breaking Bad.bw")
    }

    #[test]
    fn test_episodes() {
        let test = get_test_serie();
        assert_eq!(test.total_episodes(), 40);
    }

    #[test]
    fn test_watched() {
        let test = get_test_serie();
        assert_eq!(test.total_watched(), 10);
    }

    #[test]
    fn test_percentage() {
        let test = get_test_serie();
        assert_eq!(test.watched_percentage(), 25.);
    }

    #[test]
    fn test_display() {
        let test = get_test_serie();
        let mut buf = BufWriter::new(Vec::new());
        write!(buf, "{}", test).unwrap();
        let result = String::from_utf8(buf.into_inner().unwrap()).unwrap();
        let expected = "10/20
0/20
";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_watch() {
        let mut test = get_test_serie();
        test.watch(9);
        assert_eq!(test.current_season.unwrap(), 0);
        test.watch(11);
        assert_eq!(test.total_episodes(), 40);
        assert_eq!(test.total_watched(), 30);
        assert_eq!(test.seasons[0].watched, 20);
        assert_eq!(test.seasons[1].watched, 10);
        assert_eq!(test.current_season.unwrap(), 1);
        let left = test.watch(20);
        assert_eq!(left, 10);
        let left = test.watch(20);
        assert_eq!(left, 20);
        assert_eq!(test.seasons[1].watched, 20);
        assert_eq!(test.current_season, None);
    }

    #[test]
    fn test_watch_edge() {
        let mut test = get_test_serie();
        test.watch(10);
        assert_eq!(test.next_episode_str().unwrap().as_str(), "S02E01");
        assert_eq!(test.total_watched(), 20);
        test.watch(10);
        assert_eq!(test.next_episode_str().unwrap().as_str(), "S02E11");
        assert_eq!(test.total_watched(), 30);
        test.watch(10);
        assert_eq!(test.next_episode_str(), None);
        assert_eq!(test.total_watched(), 40);
    }

    #[test]
    fn test_unwatch_edge() {
        let mut test = get_test_serie();
        test.watch(10);
        assert_eq!(test.next_episode_str().unwrap().as_str(), "S02E01");
        assert_eq!(test.total_watched(), 20);
        test.unwatch(20);
        assert_eq!(test.next_episode_str().unwrap().as_str(), "S01E01");
        assert_eq!(test.total_watched(), 0);
    }

    #[test]
    fn test_unwatch_finished() {
        let mut test = get_test_serie();
        test.watch(30);
        assert!(test.is_finished());
        test.unwatch(20);
        assert_eq!(test.next_episode_str().unwrap().as_str(), "S02E01");
        assert_eq!(test.total_watched(), 20);
    }


    #[test]
    fn test_unwatch() {
        let mut test = get_test_serie();
        test.watch(20);
        test.unwatch(6);
        assert_eq!(test.current_season.unwrap(), 1);

        assert_eq!(test.total_episodes(), 40);
        assert_eq!(test.total_watched(), 24);
        assert_eq!(test.seasons[0].watched, 20);
        assert_eq!(test.seasons[1].watched, 4);
        test.unwatch(6);
        assert_eq!(test.current_season.unwrap(), 0);
        test.watch(6);
        assert_eq!(test.current_season.unwrap(), 1);
        test.unwatch(6);

        assert_eq!(test.seasons[0].watched, 18);
        assert_eq!(test.seasons[1].watched, 0);
    }

    #[test]
    fn test_next_episode() {
        let mut test = get_test_serie();
        let expected = "S01E11";
        assert_eq!(test.next_episode_str().unwrap(), expected);

        test.unwatch(6);
        let expected = "S01E05";
        assert_eq!(test.next_episode_str().unwrap(), expected);

        test.watch(6);
        let expected = "S01E11";
        assert_eq!(test.next_episode_str().unwrap(), expected);
    }

    #[test]
    fn test_finished() {
        let mut test: Serie = "10/20\n0/20".parse().unwrap();
        test.watch(10);
        assert!(test.is_not_finished());
        test.watch(20);
        assert!(test.is_finished());
    }

    #[test]
    fn test_merge_series_basic() {
        let mut test: Serie = "10/20\n2/20".parse().unwrap();
        test.merge_serie(&"0/20\n0/20\n0/10\n0/10".parse().unwrap());
        let expected: Serie = "10/20\n2/20\n0/10\n0/10".parse().unwrap();
        assert_eq!(test.seasons, expected.seasons)
    }

    #[test]
    fn test_merge_series_last_season_changed() {
        let mut test: Serie = "10/20\n2/20".parse().unwrap();
        test.merge_serie(&"0/20\n0/22".parse().unwrap());
        let expected: Serie = "10/20\n2/22".parse().unwrap();
        assert_eq!(test.seasons, expected.seasons)
    }
}
