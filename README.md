# icecutter

a simple frontend for ffmpeg built with iced and rust, allowing the user to cut a video and then compress it down to 10MB or less

meant for quickly cutting clips to upload to discord

<img width="638" height="509" alt="image" src="https://github.com/user-attachments/assets/7555cd24-59a7-4fae-9dd9-e9981e808b10" />

features
- the ability to cut videos down to a shorter duration
- lowering the fps/conversion to 720p for an even lower file size
- when a video file is passed in as a command line argument, that video's information is automatically inserted into all of the input fields. this allows for easy integration with right click menus and other utilities
- input validation (making sure that the user doesn't input invalid ffmpeg options)
- graphical error handling
- ffmpeg and ffprobe are bundled in
- a progress bar that tracks the ffmpeg conversion progress
