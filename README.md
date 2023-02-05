# torkoal
Baseline discord bot with minimal functionality. 
Can Join, Leave, Mute, Deafen, Unmute, Undeafen, Search and Play, Stop/Skip and Play a single YT audio source (that doesnt overlap like songbird example).

Current Implementation goals are to:
1) Create a play_looped function that will loop provided url
2) Implement functionality that will search for provided text and play first youtube search result (instead of URL, IE:
search_and_play hacker music) **COMPLETED**
3) Implement stop/skip feature to avoid having to leave and re-join bot to stop audio sources. **COMPLETED**


To Compile (On Ubuntu):

sudo apt install ffmpeg

sudo apt install youtube-dl

sudo apt install libopus-dev
