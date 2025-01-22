use core::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Season {
    pub episodes: usize,
    pub watched: usize,
}

pub struct MalformedSeason;

impl FromStr for Season {
    type Err = MalformedSeason;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sep_index = s.chars().position(|c| c == '/');
        if let Some(index) = sep_index {
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

impl fmt::Display for Season {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.watched, self.episodes)
    }
}

impl Season {
    #[inline]
    pub fn new(episodes: usize) -> Self {
        Season {
            episodes,
            ..Default::default()
        }
    }

    #[inline]
    fn not_watched(&self) -> usize {
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
