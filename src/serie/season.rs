use scanf::sscanf;

#[derive(Debug)]
pub struct Season {
    pub episodes: usize,
    pub watched: usize,
}

impl TryFrom<&str> for Season {
    type Error = &'static str;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut episodes:usize = 0;
        let mut watched:usize = 0;
        if sscanf!(value, "{}+{}", watched, episodes).is_err() {
            Err("Unable to read season")
        } else {
            Ok(Season{watched, episodes})
        }

    }
}

impl Into<String> for &Season {
    #[inline]
    fn into(self) -> String{
        format!("{}+{}", self.watched, self.episodes)
    }
}

impl Season {
    #[inline]
    pub fn new (episodes:usize) -> Self {
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
    pub fn into_string(&self) -> String {
        self.into()
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.episodes == self.watched
    }

    #[inline]
    pub fn watch(&mut self, count:usize) -> usize {
        let watch_count = count.min(self.not_watched());
        self.watched += watch_count;
        count - watch_count
    }

    #[inline]
    pub fn unwatch(&mut self, count:usize) -> usize {
        let unwatch_count = count.min(self.watched);
        self.watched -= unwatch_count;
        count - unwatch_count
    }

    #[inline]
    pub fn display(&self) -> String {
        format!("{}/{}", self.watched, self.episodes)
    }
}
