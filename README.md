# torkoal
Baseline discord bot with minimal functionality. 
Can join, leave, mute, deafen, unmute, undeafen, search and play, stop, loop and play a YT audio source.

Current Implementation goals are to:
1) Create a play_looped function that will loop provided url or search query **COMPLETED**
2) Implement functionality that will search for provided text and play first youtube search result (instead of URL, IE:
search_and_play hacker music) **COMPLETED**
3) Implement stop feature to avoid having to leave and re-join bot to stop audio sources. **COMPLETED**


Dependencies needed to compile (On Ubuntu):

sudo apt install ffmpeg

sudo apt install youtube-dl

sudo apt install libopus-dev



Dependencies needed to compile (On Arch):

pacman -Syu ffmpeg

pacman -Syu youtube-dl

pacman -Syu opus
