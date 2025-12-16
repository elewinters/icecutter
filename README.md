# icecutter

a simple frontend for ffmpeg built with iced and rust, allowing the user to cut a video and then compress it down to 10MB or less

meant for quickly cutting clips to upload to discord

<img width="639" height="528" alt="image" src="https://github.com/user-attachments/assets/0162a4dc-f7ab-4e88-bcd1-0368eb094d3e" />

features
- the ability to cut videos down to a shorter duration
- lowering the fps/resolution for an even lower file size
- automatic copying of the converted file to the clipboard for easy upload
- audio muting
- when a video file is passed in as a command line argument, that video's information is automatically inserted into all of the input fields. this allows for easy integration with right click menus and other utilities
- input validation (making sure that the user doesn't input invalid ffmpeg options)
- robust graphical error handling with OS native dialogs
- a progress bar that tracks the ffmpeg conversion progress

# installation
download icecutter from the [releases tab](https://github.com/elewinters/icecutter/releases/latest)

if you don't have ffmpeg installed on your system you may download the zip file bundle which comes with ffmpeg (make sure that icecutter and ffmpeg are always in the same directory though) 
[the executables in the bundle were downloaded from https://www.gyan.dev/ffmpeg/builds/]

if you already have it installed feel free to download the standalone versions

on linux you can easily install ffmpeg with your package manager, on a debian based distro this would look somethin like

``apt install ffmpeg``
