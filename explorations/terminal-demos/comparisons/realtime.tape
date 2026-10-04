# VHS equivalent of scenarios/realtime.toml, for comparison only (research brief, VHS
# section). Run from this directory with fdu on PATH: `vhs comparisons/realtime.tape`.
# As recorded, the non-ASCII prompt in the hidden setup line was mangled by VHS's typing,
# so the setup stayed on screen; the measurement it was kept for is unaffected.
Output out/vhs-realtime.mp4
Set Shell bash
Set FontFamily "DejaVu Sans Mono"
Set FontSize 22
Set Width 1920
Set Height 1080
Set Framerate 60
Set Padding 40
Set WindowBar Colorful
Set TypingSpeed 60ms
Set Theme "GitHub Dark"
Hide
Type `export TIMEFORMAT="real %3Rs" COLUMNS=104 PS1="❯ "; cd /; clear`
Enter
Show
Type "# 231k files, 12 GiB. Both tools warm. Nothing is sped up."
Sleep 800ms
Enter
Type "time du -sxh /"
Sleep 350ms
Enter
Sleep 1.6s
Type "time fdu / --one-filesystem --depth 1 --cache off"
Sleep 350ms
Enter
Sleep 3.5s
