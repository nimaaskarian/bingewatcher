use std::str::FromStr;

#[derive(Debug)]
pub struct Season {
    pub episodes: usize,
    pub watched: usize,
}

pub struct MalformedSeason;

impl FromStr for Season {
    type Err = MalformedSeason;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let plus_index = s.chars().position(|c| c == '+');
        if let Some(index) = plus_index {
            let watched = s[..index].parse();
            let episodes = s[index+1..].parse();
            if let (Ok(watched), Ok(episodes)) = (watched, episodes) {
                return Ok(Self {
                    episodes,
                    watched,
                })
            }
        }
        Err(MalformedSeason)
    }
}

impl From<&Season> for String {
    #[inline]
    fn from(season: &Season) -> String {
        format!("{}+{}", season.watched, season.episodes)
    }
}

impl Season {
    #[inline]
    pub fn new(episodes: usize) -> Self {
        Season {
            episodes,
            watched: 0,
        }
    }

    #[inline]
    pub fn not_watched(&self) -> usize {
        self.episodes - self.watched
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.episodes == self.watched
    }

    #[inline]
    pub fn watch(&mut self, count: usize) -> usize {
        let watch_count = count.min(self.not_watched());
        self.watched += watch_count;
        count - watch_count
    }

    #[inline]
    pub fn unwatch(&mut self, count: usize) -> usize {
        let unwatch_count = count.min(self.watched);
        self.watched -= unwatch_count;
        count - unwatch_count
    }

    #[inline]
    pub fn display(&self) -> String {
        format!("{}/{}", self.watched, self.episodes)
    }
}
