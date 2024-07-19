mod season;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    str::FromStr,
};

pub use season::Season;

pub enum SeriePrint {
    Normal,
    Extended,
    NextEpisode,
    Season,
}

#[derive(Debug)]
pub struct Serie {
    seasons: Vec<Season>,
    pub name: String,
    current_season_index: Option<usize>,
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

impl From<&Serie> for String {
    fn from(serie: &Serie) -> String {
        serie.seasons
            .iter()
            .map(|season| season.into())
            .collect::<Vec<String>>()
            .join("\n")
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
        let mut seasons = vec![];
        for line in value.to_string().lines() {
            match line.parse() {
                Err(_) => return Err(SerieParseError::ParseFailed),
                Ok(season) => seasons.push(season),
            }
        }
        if seasons.is_empty() {
            return Err(SerieParseError::EmptyFile);
        }
        Ok(Serie::new(seasons, ""))
    }
}

impl Serie {
    pub fn empty<S: AsRef<str>>(string_like: S) -> Self {
        Self::new(vec![], string_like)
    }

    pub fn new<S: AsRef<str>>(seasons: Vec<Season>, string_like: S) -> Self {
        let name = string_like.as_ref().to_string();
        Serie {
            current_season_index: Self::get_current_season_index(&seasons),
            seasons,
            name,
        }
    }

    #[inline]
    pub fn print(&self, print: &SeriePrint) {
        match print {
            SeriePrint::Extended => println!("{}", self.extended()),
            SeriePrint::NextEpisode => println!("{}", self.next_episode_str()),
            SeriePrint::Normal => println!("{}", self.display()),
            SeriePrint::Season => println!("{}", self.next_season()),
        }
    }

    #[inline]
    pub fn write(&self, path: &Path) -> io::Result<()> {
        let mut file = File::create(path)?;
        let self_str: String = self.into();
        write!(file, "{}", self_str)?;
        Ok(())
    }

    #[inline]
    pub fn write_in_dir(&self, dir: &Path) -> io::Result<()> {
        let path = dir.join(self.filename());
        self.write(&path)?;
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
    pub fn exact_matches(&self, search: &str) -> bool {
        self.name.to_lowercase() == search.to_lowercase()
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.watched() == self.episodes()
    }

    #[inline]
    fn get_current_season_index(seasons: &[Season]) -> Option<usize> {
        for (index, season) in seasons.iter().enumerate() {
            if !season.is_finished() {
                return Some(index);
            }
        }
        None
    }

    #[inline]
    fn set_current_season(&mut self) {
        self.current_season_index = Self::get_current_season_index(&self.seasons);
    }

    #[inline]
    pub fn next_season(&self) -> usize {
        self.current_season_index.unwrap() + 1
    }

    #[inline]
    pub fn next_episode(&self) -> usize {
        match self.current_season() {
            Some(season) => season.watched + 1,
            None => 1,
        }
    }

    #[inline]
    pub fn next_episode_str(&self) -> String {
        if self.is_finished() {
            String::from("FINISHED")
        } else {
            match self.current_season() {
                None => String::new(),
                Some(season) => {
                    let episode_width = number_width(season.episodes);
                    let season_width = number_width(self.seasons.len());
                    format!(
                        "S{:0season_width$}E{:0episode_width$}",
                        self.next_season(),
                        season.watched + 1
                    )
                }
            }
        }
    }

    #[inline]
    pub fn current_season(&self) -> Option<&Season> {
        let index = self.current_season_index?;
        Some(&self.seasons[index])
    }

    #[inline]
    pub fn display(&self) -> String {
        format!("{} {}", self.name, self.next_episode_str())
    }

    pub fn three_line_display(&self) -> String {
        format!(
            "Name: {}
Percentage: {:.2}%
Next episode: {}\n",
            self.name,
            self.watched_percentage(),
            self.next_episode_str()
        )
    }

    #[inline]
    pub fn extended(&self) -> String {
        self.three_line_display()
            + self.seasons
                .iter()
                .map(|season| season.display())
                .collect::<Vec<String>>()
                .join("\n")
                .as_str()
            + "\n"
    }

    #[inline]
    pub fn unwatch(&mut self, count: usize) -> usize {
        let mut unwatch_count = count;
        let season_index = self.current_season_index;

        if let Some(mut index) = season_index {
            loop {
                unwatch_count = self.seasons[index].unwatch(unwatch_count);
                if unwatch_count == 0 || index == 0 {
                    break;
                }
                index -= 1;
            }
            self.current_season_index = Some(index);
        }
        unwatch_count
    }

    #[inline]
    pub fn watch(&mut self, count: usize) -> usize {
        let mut watch_count = count;
        let season_index = self.current_season_index;

        if let Some(mut index) = season_index {
            loop {
                watch_count = self.seasons[index].watch(watch_count);
                if watch_count == 0 || index + 1 >= self.seasons.len() {
                    break;
                }
                index += 1;
            }
            self.current_season_index = Some(index);
            if self.current_season().unwrap().is_finished() {
                self.current_season_index = None;
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
        self.watched() as f32 / self.episodes() as f32 * 100.
    }

    #[inline]
    pub fn add(&mut self, season: Season) {
        self.seasons.push(season);
        self.set_current_season();
    }

    #[inline]
    fn watched(&self) -> usize {
        self.seasons.iter().map(|season| season.watched).sum()
    }

    #[inline]
    fn episodes(&self) -> usize {
        self.seasons.iter().map(|season| season.episodes).sum()
    }
}

mod tests {
    use super::*;

    fn get_test_serie() -> Serie {
        "10+20
0+20"
            .parse()
            .unwrap()
    }

    #[test]
    pub fn test_name() {
        let test = Serie::empty("Breaking Bad");
        assert_eq!(test.name, "Breaking Bad")
    }

    #[test]
    pub fn test_filename() {
        let test = Serie::empty("Breaking Bad");
        assert_eq!(test.filename(), "Breaking Bad.bw")
    }

    #[test]
    pub fn test_episodes() {
        let test = get_test_serie();
        assert_eq!(test.episodes(), 40);
    }

    #[test]
    pub fn test_watched() {
        let test = get_test_serie();
        assert_eq!(test.watched(), 10);
    }

    #[test]
    pub fn test_percentage() {
        let test = get_test_serie();
        assert_eq!(test.watched_percentage(), 25.);
    }

    #[test]
    pub fn test_into_string() {
        let test = get_test_serie();
        let result: String = (&test).into();
        let expected = "10+20
0+20";
        assert_eq!(result, expected);
    }

    #[test]
    pub fn test_add() {
        let mut test = get_test_serie();
        test.add(Season::new(40));
        assert_eq!(test.episodes(), 80);
        assert_eq!(test.watched(), 10);
    }

    #[test]
    pub fn test_watch() {
        let mut test = get_test_serie();
        test.watch(9);
        assert_eq!(test.current_season_index.unwrap(), 0);
        test.watch(11);
        assert_eq!(test.episodes(), 40);
        assert_eq!(test.watched(), 30);
        assert_eq!(test.seasons[0].watched, 20);
        assert_eq!(test.seasons[1].watched, 10);
        assert_eq!(test.current_season_index.unwrap(), 1);
        let left = test.watch(20);
        assert_eq!(left, 10);
        let left = test.watch(20);
        assert_eq!(left, 20);
        assert_eq!(test.seasons[1].watched, 20);
        assert_eq!(test.current_season_index, None);
    }

    #[test]
    pub fn test_unwatch() {
        let mut test = get_test_serie();
        test.watch(20);
        test.unwatch(6);
        assert_eq!(test.current_season_index.unwrap(), 1);

        assert_eq!(test.episodes(), 40);
        assert_eq!(test.watched(), 24);
        assert_eq!(test.seasons[0].watched, 20);
        assert_eq!(test.seasons[1].watched, 4);
        test.unwatch(6);
        assert_eq!(test.current_season_index.unwrap(), 0);
        test.watch(6);
        assert_eq!(test.current_season_index.unwrap(), 1);
        test.unwatch(6);

        assert_eq!(test.seasons[0].watched, 18);
        assert_eq!(test.seasons[1].watched, 0);
    }

    #[test]
    pub fn test_next_episode() {
        let mut test = get_test_serie();
        let expected = "S01E11";
        assert_eq!(test.next_episode_str(), expected);

        test.unwatch(6);
        let expected = "S01E05";
        assert_eq!(test.next_episode_str(), expected);

        test.watch(6);
        let expected = "S01E11";
        assert_eq!(test.next_episode_str(), expected);
    }
}
