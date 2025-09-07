# icecutter

a simple frontend for ffmpeg built with iced and rust, allowing the user to cut a video and then compress it down to 10MB or less

meant for quickly cutting clips to upload to discord

requires ffmpeg and ffprobe to be installed globally (available in the PATH)

<img width="635" height="476" alt="image" src="https://github.com/user-attachments/assets/cc3de430-5426-4518-b6ba-2eeb3cf11b16" />

features
- the ability to cut videos down to a shorter duration
- lowering the fps for an even lower file size
- conversion to 720p for an even even lower file size
- when a video file is passed in as a command line argument, that video's information is automatically inserted into all of the input fields. this allows for easy integration with right click menus and other utilities
- input validation (making sure that the user doesn't input invalid ffmpeg options)
- asynchronous file picking

features that would be nice to have but i dont rlly feel like implementing them rn
- a progress bar that shows up when a conversion is happening that tracks the ffmpeg conversion progress, this would remove the need for the terminal window to show up
- currently errors are either ignored or printed to stdout which is not very user friendly, a better solution would be to show message boxes that notify the user of the error in a graphical way
- ffmpeg is required to be installed for icecutter to work, it'd be great if the executables could be bundled in with the release binaries
- a button to select the input file with a file dialog, having to copy and paste the exact path of the video is not very user friendly
- more compression options, currently icecutter only tries to cut down the video to 10MB, as that is the file size limit for free discord accounts. however with discord nitro basic, that limit goes up to 50MB, and for nitro it goes up to 500MB. it would be nice if nitro users could set the max compression file size to 50MB or 500MB for better video quality
