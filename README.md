# bingewatcher
cli app to track your shows, using a minimalist format, and with minimum overhead.

# initialize a serie
to initialize a serie (each serie consists of
seasons. each season consists of <watched>+<total>. for example a season that
you watched 4 episodes of, and it has 10 episodes in total would be 4+10)

## initialize it using episodate api
a complex example (which I, myself use) would be like this:

```bash
bw --add-online $(bw --search-online 'breaking bad' | fzf --preview "bw --detail-online {}")
```

# watch a serie
it really depends on how you watch a serie and where you watch it from. but it
probably consists of `bw -s` (search) and `--print-mode next-episode` (which
prints the next episode you need to watch in S0xE0y format, ex: S01E01), and
`bw -a 1` which watches one episode from all the selected (or searched) series.

```bash
mpv $(fd . path/to/breaking-bad $(bw -s "breaking bad" --print-mode next-episode)) && bw -s "breaking bad" -a 1
```

# the name?
comes from the fact that you can have some command in your history to watch your
favorite show with the same command in a row, and binge-watch it.

# why not a database? like sqlite?
are you insane? that kind of overhead for something that simple?
