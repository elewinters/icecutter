# icecutter

a simple frontend for ffmpeg built with iced and rust, allowing the user to cut a video and then compress it down to 10MB or less

meant for quickly cutting clips to upload to discord

<img width="635" height="476" alt="image" src="https://github.com/user-attachments/assets/cc3de430-5426-4518-b6ba-2eeb3cf11b16" />

features
- the ability to cut videos down to a shorter duration
- lowering the fps for an even lower file size
- conversion to 720p for an even even lower file size
- when a video file is passed in as a command line argument, that video's information is automatically inserted into all of the input fields. this allows for easy integration with right click menus and other utilities
- input validation (making sure that the user doesn't input invalid ffmpeg options)
- asynchronous file picking
- graphical error handling
- ffmpeg and ffprobe are bundled in

features that would be nice to have but i dont rlly feel like implementing them rn
- a progress bar that shows up when a conversion is happening that tracks the ffmpeg conversion progress, this would remove the need for the terminal window to show up
- a button to select the input file with a file dialog, having to copy and paste the exact path of the video is not very user friendly