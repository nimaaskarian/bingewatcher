# bingewatcher
fast cli app to track your shows; using a minimalist, plain text format.

# the format
the text below defines a series, with 3 seasons. first season has 10 episodes
and all of them are watched. second season has 11 episodes and 9 of them are
watched.  season 3 has 19 episodes and none of it its watched.
```
10/10
9/11
0/19
```

# integrations
## fish abbr for adding (and updating) an online series
```fish
abbr bwo --set-cursor 'bw episodate search % | fzf --multi --preview "bw episodate detail {}" | xargs -L1 bw --print-mode extended episodate add --update'
```

## dmenu
if you have a file of links to all the series you have in `~/.series`, you can
    use the script below to integrate dmenu with bw

```bash
#!/usr/bin/env bash

name=$(bw --print-mode name | dmenu -i -p series "$@") || exit 1
path=~/.cache/bingewatcher/"$name.bw"
next_episode=$(bw --print-mode next-episode "$path")

readarray -t arr  < <(grep "/$name/.*$next_episode" ~/.series)
if [ "${#arr[@]}" == 1 ]; then
  sel=${arr[0]}
else
  sel=$(printf "%s\n" "${arr[@]}" | dmenu  -l 10 -i -p qualities "$@") || exit 1
fi
mpv "$sel" && bw watch 1 "$path"
```

# the name?
comes from the fact that you can have some command in your history to watch your
favorite show with the same command in a row, and binge-watch it.

# why not a database? like sqlite?
are you insane? that kind of overhead for something this simple?

# old version
the new version has changed in some ways. instead of arguments, most of the
arguments have either been removed (in favor of already existing apps, like
`fd` and `find` and shell file completions) or moved into subcommands.

this is commit `9aa32905e3f026ad7d89e9666d33a6ee791f6c1a` and prior. if you are
using it, check out the [old README](./OLD_README.md)
