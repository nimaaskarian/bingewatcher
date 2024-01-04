mod season;
// use std::{path::PathBuf, fs, io};
use std::{path::PathBuf, io::{self, Write}, fs::{self, File}};

use season::Season;

pub enum SeriePrint {
    Normal,
    Extended,
    NextEpisode,
}
pub struct Serie {
    seasons: Vec<Season>,
    name: String,
    current_season_index: Option<usize>,
}

#[inline(always)]
fn digit_count(mut number: u32) -> usize {
    let mut count = 0;

    while number != 0 {
        number /= 10;
        count += 1;
    }

    count
}

impl Into<String> for &Serie {
    fn into(self) -> String {
        (&self.seasons).into_iter()
        .map(|season| season.into_string())
        .collect::<Vec<String>>()
        .join("\n")
    }
}

impl TryFrom<&str> for Serie {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut seasons = vec![];
        for line in value.to_string().lines() {
            match Season::try_from(line) {
                Err(_) => return Err("Unable to create serie from string"),
                Ok(season) => seasons.push(season),
            }
        }
        if seasons.is_empty() {
            return Err("Empty file")
        }
        Ok(Serie::new(seasons, ""))
    }
}

impl TryFrom<String> for Serie {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Serie {
    pub fn empty<S: AsRef<str>>(string_like:S) -> Self {
        Self::new(vec![], string_like)
    }

    pub fn new<S: AsRef<str>>(seasons:Vec<Season>, string_like:S) -> Self {
        let name = string_like.as_ref().to_string();
        Serie {
            current_season_index: Self::get_current_season_index(&seasons),
            seasons,
            name,
        }
    }

    #[inline]
    pub fn print(&self, print:&SeriePrint) {
        match print {
            SeriePrint::Extended => println!("{}", self.extended()),
            SeriePrint::NextEpisode => println!("{}", self.next_episode()),
            SeriePrint::Normal => println!("{}", self.display()),
        }
    } 

    #[inline]
    pub fn to_string(&self) -> String {
        self.into()
    }

    #[inline]
    pub fn write(&self, path:&PathBuf) -> io::Result<()> {
        let mut file = File::create(path)?;
        write!(file,"{}", self.to_string())?;
        Ok(())
    }

    #[inline]
    pub fn write_in_dir(&self, dir:&PathBuf) -> io::Result<()> {
        let path = (&dir).join(self.filename());
        self.write(&path)?;
        Ok(())
    }

    #[inline]
    pub fn from_file(path:&PathBuf) -> Option<Self>{
        let file_content = match fs::read_to_string(path) {
            Err(_) => return None,
            Ok(content) => content,
        };

        match Self::try_from(file_content) {
            Ok(mut serie)=>{
                serie.name = path.file_stem().unwrap().to_str().unwrap().to_string();

                return Some(serie)
            },
            Err(_)=>return None,
        }
    }

    #[inline]
    pub fn matches(&self, search:&String) -> bool {
        self.name.to_lowercase().contains(&search.to_lowercase())
    }

    #[inline]
    pub fn exact_matches(&self, search:&String) -> bool {
        self.name.to_lowercase() == search.to_lowercase()
    }
    
    #[inline]
    pub fn finished(&self) -> bool{
        self.watched() == self.episodes()
    }

    #[inline]
    fn get_current_season_index(seasons:&Vec<Season>) -> Option<usize>{
        for (index, season) in seasons.into_iter().enumerate() {
            if !season.is_finished() {
                return Some(index);
            }
        }
        return None;
    }

    #[inline]
    fn set_current_season(&mut self) {
        self.current_season_index =  Self::get_current_season_index(&self.seasons);
    }

    #[inline]
    pub fn next_episode(&self) -> String {
        if self.finished() {
            String::new()
        } else {
            match self.current_season() {
                None => String::new(),
                Some(season) => {
                    let episode_width = match digit_count(season.episodes) {
                        0 | 1 => 2,
                        any => any,
                    };
                    let season_width = match digit_count(self.seasons.len() as u32) {
                        0 | 1 => 2,
                        any => any,
                    };
                    format!("S{:0season_width$}E{:0episode_width$}",self.current_season_index.unwrap()+1,season.watched+1)
                }
            }
        }
    }

    #[inline]
    pub fn current_season(&self) -> Option<&Season> {
        match self.current_season_index {
            Some(index) => Some(&self.seasons[index]),
            None=>None
        }
    }

    #[inline]
    pub fn display(&self) -> String {
        format!("{} {:.2}%", self.name, self.watched_percentage())
    }

    #[inline]
    pub fn extended(&self) -> String {
        format!("{} {:.2}%\n", self.name, self.watched_percentage())
        + (&self.seasons).into_iter()
        .map(|season| season.display())
        .collect::<Vec<String>>()
        .join("\n").as_str() + "\n"
    }

    #[inline]
    pub fn unwatch(&mut self, count:u32) -> u32{
        let mut unwatch_count = count;
        let season_index = self.current_season_index;

        if let Some(mut index) = season_index {
            loop {
                unwatch_count = self.seasons[index].unwatch(unwatch_count);
                if unwatch_count > 0 && index != 0 {
                    index-=1;
                } else {
                    break;
                }
            }
            self.current_season_index = Some(index);
        }
        return unwatch_count
    }

    #[inline]
    pub fn watch(&mut self, count:u32) -> u32{
        let mut watch_count = count;
        let season_index = self.current_season_index;

        if let Some(mut index) = season_index {
            loop {
                watch_count = self.seasons[index].watch(watch_count);
                if watch_count > 0 && index+1 < self.seasons.len() {
                    index+=1;
                    self.current_season_index = Some(index);
                } else {
                    break;
                }
            }
            if self.current_season().unwrap().is_finished() {
                self.current_season_index = None;
            }
        }
        return watch_count
    }

    #[inline]
    pub fn filename(&self) -> String {
        format!("{}.bw", self.name)
    }

    #[inline]
    pub fn watched_percentage(&self) -> f32 {
        self.watched()as f32/self.episodes() as f32 * 100.
    }
    
    #[inline]
    pub fn add(&mut self, season:Season) {
        self.seasons.push(season);
        self.set_current_season();
    }

    #[inline]
    fn watched(&self) -> u32 {
        self.seasons.iter().map(|season| season.watched).sum()
    }

    #[inline]
    fn episodes(&self) -> u32 {
        self.seasons.iter().map(|season| season.episodes).sum()
    }
}

mod tests {
    use super::*;

    fn get_test_serie() -> Serie{
                Serie::try_from("10+20
0+20").unwrap()

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
        let result: String= (&test).into();
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
        test.watch(20);
        assert_eq!(test.episodes(), 40);
        assert_eq!(test.watched(), 30);
        assert_eq!(test.seasons[0].watched, 20);
        assert_eq!(test.seasons[1].watched, 10);
        assert_eq!(test.current_season_index.unwrap(), 1);
        let left = test.watch(20);
        assert_eq!(left, 10);
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
        assert_eq!(test.next_episode(), expected);

        test.unwatch(6);
        let expected = "S01E05";
        assert_eq!(test.next_episode(), expected);

        test.watch(6);
        let expected = "S01E11";
        assert_eq!(test.next_episode(), expected);
    }
}
