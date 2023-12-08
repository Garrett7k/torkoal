# torkoal
Discord bot with minimal functionality. Breaks often.



Use at your own will:

Create initial bot in discord developer portal. 

Create your own token and add to system environment. 

Typically able to set DISCORD_TOKEN env variable in /etc/environment for persistence. 

Dependencies needed to compile (On Ubuntu):

sudo apt install ffmpeg

sudo apt install youtube-dl

sudo apt install libopus-dev 

Follow the below step for Youtube-dl extractor fix.



Dependencies needed to compile (On Arch):

sudo pacman -Syu ffmpeg

sudo pacman -Syu youtube-dl (No longer works, you have to install youtube-dl via pip. sudo pacman -Syu python sudo pacman -Syu pip)

Once pip is installed and you confirmed python3.11 is installed, install youtube-dl via pip.

sudo pip install youtube_dl --user --break-system-packages

sudo pacman -Syu opus

You will need to edit line 1794 in the youtube-dl extractor file youtube.py (pip installs in /home/user/.local/lin/python3.11/site-packages/youtube_dl/extractor/youtube.py) 
This resolves the extrator uploader ID error. 
